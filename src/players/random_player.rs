use rand::{rngs::StdRng, seq::SliceRandom};
use std::fmt::Debug;

use crate::{actions::Action, Deck, State};

use super::Player;

pub struct RandomPlayer {
    pub deck: Deck,
}

impl Player for RandomPlayer {
    fn decision_fn(
        &mut self,
        rng: &mut StdRng,
        _: &State,
        possible_actions: Vec<Action>,
    ) -> Action {
        possible_actions
            .choose(rng)
            .expect("There should always be at least one playable action")
            .clone()
    }

    fn get_deck(&self) -> Deck {
        self.deck.clone()
    }
}

impl Debug for RandomPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomPlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        actions::{Action, SimpleAction},
        card_ids::CardId,
        database::get_card_by_enum,
        test_helpers::load_test_decks,
        types::{Card, EnergyType},
    };
    use rand::SeedableRng;

    #[test]
    fn test_random_player_creation() {
        let (deck, _) = load_test_decks();
        let player = RandomPlayer { deck: deck.clone() };
        
        assert_eq!(player.get_deck().cards.len(), deck.cards.len());
    }

    #[test]
    fn test_random_player_single_action() {
        let (deck, _) = load_test_decks();
        let mut player = RandomPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create a single possible action
        let action = Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        };
        let possible_actions = vec![action.clone()];
        
        // Should always return the only available action
        let chosen = player.decision_fn(&mut rng, &state, possible_actions);
        assert_eq!(chosen, action);
    }

    #[test]
    fn test_random_player_multiple_actions() {
        let (deck, _) = load_test_decks();
        let mut player = RandomPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create multiple possible actions
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::DrawCard,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::Attach {
                    attachments: vec![(1, EnergyType::Grass, 0)],
                    is_turn_energy: true,
                },
                is_stack: false,
            },
        ];
        
        // Track which actions are chosen over multiple runs
        let mut chosen_counts = std::collections::HashMap::new();
        for _ in 0..100 {
            let chosen = player.decision_fn(&mut rng, &state, actions.clone());
            *chosen_counts.entry(format!("{:?}", chosen.action)).or_insert(0) += 1;
        }
        
        // All actions should be chosen at least once with high probability
        assert!(chosen_counts.len() > 1, "Random player should choose different actions");
        
        // No action should dominate (rough check for randomness)
        for count in chosen_counts.values() {
            assert!(*count < 80, "Random selection seems biased");
        }
    }

    #[test]
    #[should_panic(expected = "There should always be at least one playable action")]
    fn test_random_player_no_actions_panics() {
        let (deck, _) = load_test_decks();
        let mut player = RandomPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Empty action list should panic
        let possible_actions = vec![];
        player.decision_fn(&mut rng, &state, possible_actions);
    }

    #[test]
    fn test_random_player_deterministic_with_seed() {
        let (deck, _) = load_test_decks();
        let mut player1 = RandomPlayer { deck: deck.clone() };
        let mut player2 = RandomPlayer { deck };
        
        let state = State::new(&player1.get_deck(), &player2.get_deck());
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::DrawCard,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::Place(
                    get_card_by_enum(CardId::A1001Bulbasaur),
                    0
                ),
                is_stack: false,
            },
        ];
        
        // Same seed should produce same choices
        let mut rng1 = StdRng::seed_from_u64(12345);
        let mut rng2 = StdRng::seed_from_u64(12345);
        
        let choice1 = player1.decision_fn(&mut rng1, &state, actions.clone());
        let choice2 = player2.decision_fn(&mut rng2, &state, actions.clone());
        
        assert_eq!(choice1, choice2);
    }

    #[test]
    fn test_random_player_debug_format() {
        let (deck, _) = load_test_decks();
        let player = RandomPlayer { deck };
        
        let debug_str = format!("{:?}", player);
        assert_eq!(debug_str, "RandomPlayer");
    }

    #[test]
    fn test_random_player_get_deck() {
        let (deck, _) = load_test_decks();
        let original_deck_size = deck.cards.len();
        let player = RandomPlayer { deck };
        
        let retrieved_deck = player.get_deck();
        assert_eq!(retrieved_deck.cards.len(), original_deck_size);
        
        // Verify deck is cloned, not moved
        let second_retrieval = player.get_deck();
        assert_eq!(second_retrieval.cards.len(), original_deck_size);
    }

    #[test]
    fn test_random_player_ignores_game_state() {
        let (deck, _) = load_test_decks();
        let mut player = RandomPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        
        // Create different game states
        let state1 = State::new(&player.get_deck(), &player.get_deck());
        let mut state2 = State::new(&player.get_deck(), &player.get_deck());
        state2.turn_count = 50;
        state2.points = [3, 3];
        
        let action = Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        };
        
        // Should make same decision regardless of state (with same RNG)
        let mut rng1 = StdRng::seed_from_u64(100);
        let mut rng2 = StdRng::seed_from_u64(100);
        
        let choice1 = player.decision_fn(&mut rng1, &state1, vec![action.clone()]);
        let choice2 = player.decision_fn(&mut rng2, &state2, vec![action.clone()]);
        
        assert_eq!(choice1, choice2);
    }
}
