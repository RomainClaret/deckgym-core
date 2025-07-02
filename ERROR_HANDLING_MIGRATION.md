# Error Handling Migration Guide

## Overview

This guide helps migrate from panic-prone code (using `unwrap()`, `expect()`) to proper error handling using the new `GameError` enum and `GameResult` type.

## Key Changes

### 1. New Error Types

```rust
use deckgym::{GameError, GameResult};

// GameResult<T> is an alias for Result<T, GameError>
pub type GameResult<T> = Result<T, GameError>;
```

### 2. Safe Method Variants

For every panic-prone method, there's now a safe variant that returns `Result`:

| Original Method | Safe Method | 
|----------------|-------------|
| `state.get_remaining_hp(p, i)` | `state.get_remaining_hp_safe(p, i)?` |
| `state.remove_card_from_hand(p, c)` | `state.remove_card_from_hand_safe(p, c)?` |
| `state.get_active(p)` | `state.get_active_safe(p)?` |
| `state.get_active_mut(p)` | `state.get_active_mut_safe(p)?` |
| `state.generate_energy()` | `state.generate_energy_safe()?` |

## Migration Examples

### Example 1: Simple Migration

**Before:**
```rust
fn attack_damage(state: &State, player: usize, position: usize) -> u32 {
    let pokemon = &state.in_play_pokemon[player][position]
        .as_ref()
        .expect("Pokemon should be there");
    pokemon.remaining_hp
}
```

**After:**
```rust
fn attack_damage(state: &State, player: usize, position: usize) -> GameResult<u32> {
    let pokemon = state.get_pokemon_safe(player, position)?;
    Ok(pokemon.remaining_hp)
}
```

### Example 2: Error Propagation

**Before:**
```rust
fn evolve_pokemon(state: &mut State, player: usize, card: &Card, pos: usize) {
    let old = state.in_play_pokemon[player][pos]
        .as_ref()
        .unwrap();
    
    // ... evolution logic ...
    
    state.remove_card_from_hand(player, card);
}
```

**After:**
```rust
fn evolve_pokemon(state: &mut State, player: usize, card: &Card, pos: usize) -> GameResult<()> {
    let old = state.get_pokemon_safe(player, pos)?;
    
    // ... evolution logic ...
    
    state.remove_card_from_hand_safe(player, card)?;
    Ok(())
}
```

### Example 3: Custom Error Messages

**Before:**
```rust
let energy_cost = pokemon.attack_cost
    .get(attack_idx)
    .expect("Invalid attack index");
```

**After:**
```rust
let energy_cost = pokemon.attack_cost
    .get(attack_idx)
    .ok_or_else(|| GameError::InvalidAction {
        action: "Attack".to_string(),
        reason: format!("Attack index {} out of bounds", attack_idx),
    })?;
```

## Error Handling Patterns

### Pattern 1: Using `ok_or_game_error`

```rust
use deckgym::errors::OptionExt;

// Convert Option to Result with context
let card = deck.cards
    .first()
    .ok_or_game_error(|| GameError::EmptyDeck { player })?;
```

### Pattern 2: Match on Specific Errors

```rust
match state.get_active_safe(player) {
    Ok(pokemon) => {
        // Use pokemon
    }
    Err(GameError::NoActivePokemon { .. }) => {
        // Handle no active Pokemon
    }
    Err(e) => return Err(e),
}
```

### Pattern 3: Error Recovery

```rust
// Try primary action, fall back on error
let hp = state.get_remaining_hp_safe(player, 0)
    .unwrap_or(0); // Default to 0 if no Pokemon
```

## Common Error Types

### State Errors
- `InvalidCardPosition`: Position out of bounds
- `CardNotInHand`: Card not found in player's hand  
- `NoPokemonAtPosition`: No Pokemon at specified position
- `NoActivePokemon`: No active Pokemon (position 0)
- `EmptyDeck`: Deck has no cards

### Action Errors
- `InvalidAction`: Generic invalid action with reason
- `IllegalMove`: Move violates game rules
- `InvalidEvolution`: Evolution requirements not met
- `InvalidAttachment`: Can't attach energy/tool

### Game State Errors
- `GameAlreadyOver`: Trying to act after game ended
- `InvalidGameState`: Game in inconsistent state
- `NoLegalMoves`: Player has no valid moves

## Step-by-Step Migration

### Step 1: Update Function Signatures

Add `-> GameResult<T>` to functions that can fail:

```rust
// Before
fn do_action(state: &mut State) {
    // ...
}

// After  
fn do_action(state: &mut State) -> GameResult<()> {
    // ...
    Ok(())
}
```

### Step 2: Replace Panicking Calls

Find and replace:
- `.unwrap()` → `?`
- `.expect("msg")` → `.ok_or_else(|| GameError::...)?`
- `panic!("msg")` → `return Err(GameError::...)`

### Step 3: Handle Errors at Boundaries

At the top level (main game loop, AI decisions), handle errors:

```rust
// In game loop
match apply_action(&mut state, &action) {
    Ok(()) => continue,
    Err(GameError::IllegalMove { .. }) => {
        // Skip illegal move, try another
    }
    Err(e) => {
        log::error!("Game error: {}", e);
        break;
    }
}
```

### Step 4: Add Context

Use error helper functions for better messages:

```rust
// Generic error
Err(GameError::InvalidAction {
    action: "Retreat".to_string(),
    reason: "Insufficient energy".to_string(),
})

// Better - use specific error
Err(GameError::MissingEnergy {
    required: vec!["Fighting".to_string()],
    available: vec![],
})
```

## Testing Error Cases

### Unit Tests

```rust
#[test]
fn test_error_handling() {
    let mut state = State::new(&deck1, &deck2);
    
    // Test specific error
    let result = state.get_active_safe(0);
    assert!(matches!(result, Err(GameError::NoActivePokemon { .. })));
}
```

### Integration Tests

```rust
#[test]
fn test_game_handles_errors() {
    let mut game = Game::new(players, seed);
    
    // Game should handle errors gracefully
    let outcome = game.play();
    assert!(outcome.is_some() || game.state.turn_count >= 100);
}
```

## Best Practices

1. **Fail Fast**: Return errors early rather than continuing with bad state
2. **Be Specific**: Use specific error variants rather than generic ones
3. **Add Context**: Include relevant information in error messages
4. **Log Errors**: Log errors at appropriate levels for debugging
5. **Test Errors**: Write tests for error conditions, not just happy paths

## Performance Considerations

- Error handling adds minimal overhead compared to panicking
- `Result` is optimized by the compiler (often zero-cost)
- Errors should be exceptional - hot paths should rarely error

## Gradual Migration

You don't need to migrate everything at once:

1. Start with critical paths (game loop, state mutations)
2. Add safe variants alongside unsafe ones
3. Gradually deprecate unsafe methods
4. Eventually remove unsafe methods

## Compatibility

The safe methods can coexist with original panicking methods:

```rust
// Both available during migration
state.get_active(0);           // Panics on error
state.get_active_safe(0)?;     // Returns Result
```

This allows gradual migration without breaking existing code.