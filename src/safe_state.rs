use crate::{
    errors::{GameError, GameResult, OptionExt},
    state::State,
    types::{Card, PlayedCard},
};

/// Safe extension methods for State that return Results instead of panicking
impl State {
    /// Safely get remaining HP of a Pokemon
    pub fn get_remaining_hp_safe(&self, player: usize, index: usize) -> GameResult<u32> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        if index >= 4 {
            return Err(GameError::invalid_position(index, 3));
        }
        
        self.in_play_pokemon[player][index]
            .as_ref()
            .map(|p| p.remaining_hp)
            .ok_or_game_error(|| GameError::no_pokemon(player, index))
    }
    
    /// Safely remove a card from hand
    pub fn remove_card_from_hand_safe(&mut self, player: usize, card: &Card) -> GameResult<()> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        let card_name = match card {
            Card::Pokemon(p) => &p.name,
            Card::Trainer(t) => &t.name,
        };
        
        let index = self.hands[player]
            .iter()
            .position(|x| x == card)
            .ok_or_game_error(|| GameError::card_not_in_hand(card_name, player))?;
            
        self.hands[player].swap_remove(index);
        Ok(())
    }
    
    /// Safely discard a card from hand
    pub fn discard_card_from_hand_safe(&mut self, player: usize, card: &Card) -> GameResult<()> {
        self.remove_card_from_hand_safe(player, card)?;
        self.discard_piles[player].push(card.clone());
        Ok(())
    }
    
    /// Safely get the active Pokemon
    pub fn get_active_safe(&self, player: usize) -> GameResult<&PlayedCard> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        self.in_play_pokemon[player][0]
            .as_ref()
            .ok_or_game_error(|| GameError::no_active(player))
    }
    
    /// Safely get mutable reference to active Pokemon
    pub fn get_active_mut_safe(&mut self, player: usize) -> GameResult<&mut PlayedCard> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        self.in_play_pokemon[player][0]
            .as_mut()
            .ok_or_game_error(|| GameError::no_active(player))
    }
    
    /// Safely get a Pokemon at any position
    pub fn get_pokemon_safe(&self, player: usize, position: usize) -> GameResult<&PlayedCard> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        if position >= 4 {
            return Err(GameError::invalid_position(position, 3));
        }
        
        self.in_play_pokemon[player][position]
            .as_ref()
            .ok_or_game_error(|| GameError::no_pokemon(player, position))
    }
    
    /// Safely get mutable reference to a Pokemon at any position
    pub fn get_pokemon_mut_safe(&mut self, player: usize, position: usize) -> GameResult<&mut PlayedCard> {
        if player >= 2 {
            return Err(GameError::InvalidPlayer { player });
        }
        
        if position >= 4 {
            return Err(GameError::invalid_position(position, 3));
        }
        
        self.in_play_pokemon[player][position]
            .as_mut()
            .ok_or_game_error(|| GameError::no_pokemon(player, position))
    }
    
    /// Safely check if game can continue
    pub fn validate_game_state(&self) -> GameResult<()> {
        if self.is_game_over() {
            return Err(GameError::GameAlreadyOver);
        }
        
        // Check both players have at least one Pokemon
        let player1_has_pokemon = self.in_play_pokemon[0].iter().any(|p| p.is_some());
        let player2_has_pokemon = self.in_play_pokemon[1].iter().any(|p| p.is_some());
        
        if !player1_has_pokemon || !player2_has_pokemon {
            return Err(GameError::InvalidGameState {
                description: "At least one player has no Pokemon in play".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Safely generate energy (returns error if deck has no energy types)
    pub fn generate_energy_safe(&mut self) -> GameResult<()> {
        if self.current_player >= 2 {
            return Err(GameError::InvalidPlayer { player: self.current_player });
        }
        
        let deck_energies = &self.decks[self.current_player].energy_types;
        
        if deck_energies.is_empty() {
            return Err(GameError::InvalidDeckFormat {
                reason: "Deck has no energy types defined".to_string(),
            });
        }
        
        if deck_energies.len() == 1 {
            self.current_energy = Some(deck_energies[0]);
            return Ok(());
        }
        
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let generated = deck_energies
            .choose(&mut rng)
            .ok_or_game_error(|| GameError::internal("generate_energy", "Failed to choose random energy"))?;
            
        self.current_energy = Some(*generated);
        Ok(())
    }
}

/// Safe wrapper functions that can be used as drop-in replacements
pub mod safe_operations {
    use super::*;
    use crate::{
        actions::Action,
        types::EnergyType,
    };
    
    /// Safe version of apply_evolve
    pub fn apply_evolve_safe(
        acting_player: usize,
        state: &mut State,
        card: &Card,
        position: usize,
    ) -> GameResult<()> {
        use crate::hooks::to_playable_card;
        
        // Validate inputs
        if acting_player >= 2 {
            return Err(GameError::InvalidPlayer { player: acting_player });
        }
        
        if position >= 4 {
            return Err(GameError::invalid_position(position, 3));
        }
        
        // Get the card to evolve
        let pokemon_card = match card {
            Card::Pokemon(p) => {
                if p.stage == 0 {
                    return Err(GameError::InvalidEvolution {
                        reason: "Cannot evolve with a basic Pokemon".to_string(),
                    });
                }
                p
            }
            Card::Trainer(_) => {
                return Err(GameError::InvalidEvolution {
                    reason: "Cannot evolve with a trainer card".to_string(),
                });
            }
        };
        
        // Get the old Pokemon
        let old_pokemon = state.get_pokemon_safe(acting_player, position)?;
        
        // Validate evolution chain
        if let Some(evolves_from) = &pokemon_card.evolves_from {
            if evolves_from != &old_pokemon.get_name() {
                return Err(GameError::InvalidEvolution {
                    reason: format!(
                        "{} doesn't evolve from {}",
                        pokemon_card.name,
                        old_pokemon.get_name()
                    ),
                });
            }
        }
        
        // Create evolved Pokemon
        let mut played_card = to_playable_card(card, true);
        let damage_taken = old_pokemon.total_hp - old_pokemon.remaining_hp;
        played_card.remaining_hp = played_card.remaining_hp.saturating_sub(damage_taken);
        played_card.attached_energy = old_pokemon.attached_energy.clone();
        played_card.cards_behind = old_pokemon.cards_behind.clone();
        played_card.cards_behind.push(old_pokemon.card.clone());
        
        // Place evolved Pokemon
        state.in_play_pokemon[acting_player][position] = Some(played_card);
        
        // Remove from hand
        state.remove_card_from_hand_safe(acting_player, card)?;
        
        Ok(())
    }
    
    /// Safe version of apply_retreat
    pub fn apply_retreat_safe(
        acting_player: usize,
        state: &mut State,
        bench_idx: usize,
        is_free: bool,
    ) -> GameResult<()> {
        use crate::hooks::get_retreat_cost;
        
        // Validate inputs
        if acting_player >= 2 {
            return Err(GameError::InvalidPlayer { player: acting_player });
        }
        
        if bench_idx >= 4 || bench_idx == 0 {
            return Err(GameError::InvalidAction {
                action: "Retreat".to_string(),
                reason: "Can only retreat to bench positions 1-3".to_string(),
            });
        }
        
        // Ensure bench Pokemon exists
        state.get_pokemon_safe(acting_player, bench_idx)?;
        
        if !is_free {
            // Check retreat cost
            let active = state.get_active_safe(acting_player)?;
            let retreat_cost = get_retreat_cost(state, active);
            
            let active_mut = state.get_active_mut_safe(acting_player)?;
            let attached_energy = &mut active_mut.attached_energy;
            
            if attached_energy.len() < retreat_cost.len() {
                return Err(GameError::MissingEnergy {
                    required: retreat_cost.iter().map(|e| format!("{:?}", e)).collect(),
                    available: attached_energy.iter().map(|e| format!("{:?}", e)).collect(),
                });
            }
            
            // Discard energy for retreat cost
            let count = retreat_cost.len();
            attached_energy.truncate(attached_energy.len() - count);
        }
        
        // Swap Pokemon
        state.in_play_pokemon[acting_player].swap(0, bench_idx);
        
        // Cure status conditions on benched Pokemon
        if let Some(pokemon) = &state.in_play_pokemon[acting_player][bench_idx] {
            state.in_play_pokemon[acting_player][bench_idx] = Some(PlayedCard {
                poisoned: false,
                paralyzed: false,
                asleep: false,
                ..pokemon.clone()
            });
        }
        
        state.has_retreated = true;
        
        Ok(())
    }
    
    /// Safe version of apply_healing
    pub fn apply_healing_safe(
        acting_player: usize,
        state: &mut State,
        position: usize,
        amount: u32,
    ) -> GameResult<()> {
        let pokemon = state.get_pokemon_mut_safe(acting_player, position)?;
        pokemon.heal(amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_helpers::load_test_decks,
        types::{Pokemon, TrainerCard, TrainerType},
    };
    
    #[test]
    fn test_safe_get_remaining_hp() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        // Should fail - no Pokemon placed
        assert!(state.get_remaining_hp_safe(0, 0).is_err());
        
        // Should fail - invalid player
        assert!(state.get_remaining_hp_safe(2, 0).is_err());
        
        // Should fail - invalid position
        assert!(state.get_remaining_hp_safe(0, 4).is_err());
    }
    
    #[test]
    fn test_safe_remove_card() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let fake_card = Card::Trainer(TrainerCard {
            id: 999,
            name: "Fake Card".to_string(),
            trainer_type: TrainerType::Item,
        });
        
        // Should fail - card not in hand
        assert!(state.remove_card_from_hand_safe(0, &fake_card).is_err());
        
        // Should fail - invalid player
        assert!(state.remove_card_from_hand_safe(2, &fake_card).is_err());
    }
    
    #[test]
    fn test_safe_generate_energy() {
        let (mut deck_a, deck_b) = load_test_decks();
        deck_a.energy_types.clear();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Should fail - no energy types
        assert!(state.generate_energy_safe().is_err());
        
        // Fix and retry
        state.decks[0].energy_types.push(crate::types::EnergyType::Grass);
        assert!(state.generate_energy_safe().is_ok());
        assert_eq!(state.current_energy, Some(crate::types::EnergyType::Grass));
    }
}