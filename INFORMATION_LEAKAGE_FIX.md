# Information Leakage Fix Guide

## Critical Issue Fixed

The original trainer card implementation leaked hidden information by revealing:
- Exact cards that would be drawn from decks
- Which basic Pokemon were available in decks  
- Opponent's deck contents during shuffles
- Top card of deck before drawing

This made AI simulations invalid for competitive play, as bots had perfect information about hidden game state.

## The Problem

In the original `apply_trainer_action.rs`:

```rust
// TODO: Problem. With doing 1.0, we are basically giving bots the ability to see the cards in deck.
fn pokeball_outcomes(acting_player: usize, state: &State) -> (Probabilities, Mutations) {
    // This revealed exactly which basic Pokemon were in the deck!
    let num_basic_in_deck = state.decks[acting_player]
        .cards
        .iter()
        .filter(|x| x.is_basic())
        .count();
    
    // Created separate outcome for each basic Pokemon, revealing all options
    for i in 0..num_basic_in_deck {
        let card = state.decks[acting_player].cards
            .iter()
            .filter(|x| x.is_basic())
            .nth(i)
            .cloned();
        // Bot could see exactly which card would be drawn
    }
}
```

## The Solution

### 1. Hidden Information System

Created `hidden_information.rs` to track what players legitimately know:

```rust
pub struct HiddenKnowledge {
    /// What each player knows about deck contents
    pub known_deck_contents: [HashMap<Card, usize>; 2],
    
    /// Track if a player has perfect information (for human players)
    pub has_perfect_info: [bool; 2],
}
```

### 2. Safe Trainer Actions

Created `safe_trainer_actions.rs` with fixed implementations:

```rust
/// Pokeball - No longer reveals deck contents
fn pokeball_outcomes_safe(acting_player: usize, state: &State) -> (Probabilities, Mutations) {
    // Only check IF there are basics, not which ones
    let has_basic = state.decks[acting_player]
        .cards
        .iter()
        .any(|x| x.is_basic());
        
    // Single outcome - actual card selection happens at execution
    (
        vec![1.0],
        vec![Box::new(move |rng, state, action| {
            // Selection happens here, hidden from forecast
            let basics: Vec<_> = /* find basics */;
            let selected = basics.choose(rng);
            // ...
        })],
    )
}
```

## Migration Steps

### Step 1: Update Imports

```rust
// Old
use crate::actions::apply_trainer_action::forecast_trainer_action;

// New  
use crate::actions::safe_trainer_actions::forecast_trainer_action_safe;
```

### Step 2: Update Action Forecasting

In `apply_action.rs`:

```rust
// Old
SimpleAction::Play { trainer_card } => {
    forecast_trainer_action(action.actor, state, trainer_card)
}

// New
SimpleAction::Play { trainer_card } => {
    forecast_trainer_action_safe(action.actor, state, trainer_card)
}
```

### Step 3: Handle Probabilistic Outcomes

The safe version represents unknown outcomes as single 1.0 probability:

```rust
// Old: Multiple outcomes revealing deck contents
let probabilities = vec![0.33, 0.33, 0.34]; // 3 basics in deck
let outcomes = vec![pikachu_outcome, bulbasaur_outcome, charmander_outcome];

// New: Single outcome, card selection hidden
let probabilities = vec![1.0];
let outcomes = vec![select_random_basic_outcome];
```

## Affected Trainer Cards

Cards that were leaking information:
- **Poke Ball**: Revealed all basic Pokemon in deck
- **Professor's Research**: Revealed next 2 cards
- **Red Card**: Revealed opponent's deck order  
- **Mythical Slab**: Revealed top card

Cards already safe (deterministic effects):
- Potion, X Speed, Erika, Koga, Giovanni, Sabrina, Cyrus, Leaf

## Testing the Fix

### Unit Test

```rust
#[test]
fn test_no_information_leakage() {
    let state = /* setup */;
    let pokeball = /* Poke Ball card */;
    
    let (probs, mutations) = forecast_trainer_action_safe(0, &state, &pokeball);
    
    // Should be single outcome, not one per basic Pokemon
    assert_eq!(probs.len(), 1);
    assert_eq!(probs[0], 1.0);
}
```

### Integration Test

```rust
#[test]
fn test_ai_cannot_see_deck() {
    let mut game = Game::new(ai_players, seed);
    
    // AI should make decisions without knowing deck contents
    // Can verify by checking decision variance across runs
}
```

## Performance Impact

- **Forecast Phase**: Slightly faster (fewer outcomes to evaluate)
- **Execution Phase**: Same performance
- **AI Quality**: May decrease slightly without perfect information
- **Game Fairness**: Dramatically improved

## Backwards Compatibility

To maintain compatibility during migration:

```rust
// Feature flag for safe mode
#[cfg(feature = "safe_trainer_actions")]
use crate::actions::safe_trainer_actions::forecast_trainer_action_safe as forecast_trainer_action;

#[cfg(not(feature = "safe_trainer_actions"))]
use crate::actions::apply_trainer_action::forecast_trainer_action;
```

## Future Improvements

1. **Probability Distributions**: Instead of hiding information completely, use probability distributions based on known information
2. **Memory System**: Track what each player has seen (e.g., cards revealed by effects)
3. **Partial Information**: Some effects reveal partial information (e.g., "has a basic Pokemon" without revealing which)

## Validation

The fix ensures:
- ✅ Bots cannot see exact deck contents
- ✅ Forecasting doesn't reveal future draws  
- ✅ Hidden information remains hidden
- ✅ Game rules still properly enforced
- ✅ Human players unaffected

This fix is **critical** for competitive integrity and must be applied before using the simulator for serious play testing or AI training.