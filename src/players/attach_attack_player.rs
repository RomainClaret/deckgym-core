use rand::rngs::StdRng;
use std::fmt::Debug;

use crate::{
    actions::{Action, SimpleAction},
    Deck, State,
};

use super::Player;

/// A player that always tries to Attach energy to active Pokemon
///   if it can. If it can't, it will attack with the active Pokemon.
/// Else it will just do the first possible action.
pub struct AttachAttackPlayer {
    pub deck: Deck,
}

impl Player for AttachAttackPlayer {
    fn decision_fn(&mut self, _: &mut StdRng, _: &State, possible_actions: Vec<Action>) -> Action {
        let maybe_attach = possible_actions
            .iter()
            .find(|action| matches!(action.action, SimpleAction::Attach { .. }));
        if let Some(attach) = maybe_attach {
            return attach.clone();
        }
        let maybe_attack = possible_actions
            .iter()
            .find(|action| matches!(action.action, SimpleAction::Attack(_)));
        if let Some(attack) = maybe_attack {
            return attack.clone();
        }
        possible_actions
            .first()
            .expect("There should always be at least one playable action")
            .clone()
    }

    fn get_deck(&self) -> Deck {
        self.deck.clone()
    }
}

impl Debug for AttachAttackPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttachAttackPlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        card_ids::CardId,
        database::get_card_by_enum,
        test_helpers::load_test_decks,
        types::EnergyType,
    };
    use rand::SeedableRng;

    #[test]
    fn test_attach_attack_player_creation() {
        let (deck, _) = load_test_decks();
        let player = AttachAttackPlayer { deck: deck.clone() };
        
        assert_eq!(player.get_deck().cards.len(), deck.cards.len());
    }

    #[test]
    fn test_prioritizes_attach_action() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create actions with attach as second option
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
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
            Action {
                actor: 0,
                action: SimpleAction::DrawCard,
                is_stack: false,
            },
        ];
        
        let chosen = player.decision_fn(&mut rng, &state, actions);
        
        // Should choose attach action
        match chosen.action {
            SimpleAction::Attach { .. } => {},
            _ => panic!("Expected Attach action to be chosen"),
        }
    }

    #[test]
    fn test_prioritizes_attack_when_no_attach() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create actions with attack but no attach
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::Attack(0),
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::DrawCard,
                is_stack: false,
            },
        ];
        
        let chosen = player.decision_fn(&mut rng, &state, actions);
        
        // Should choose attack action
        match chosen.action {
            SimpleAction::Attack(_) => {},
            _ => panic!("Expected Attack action to be chosen"),
        }
    }

    #[test]
    fn test_chooses_first_action_as_fallback() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create actions without attach or attack
        let first_action = Action {
            actor: 0,
            action: SimpleAction::DrawCard,
            is_stack: false,
        };
        let actions = vec![
            first_action.clone(),
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
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
        
        let chosen = player.decision_fn(&mut rng, &state, actions);
        
        // Should choose first action
        assert_eq!(chosen, first_action);
    }

    #[test]
    fn test_multiple_attach_actions_chooses_first() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create multiple attach actions
        let first_attach = Action {
            actor: 0,
            action: SimpleAction::Attach {
                attachments: vec![(1, EnergyType::Grass, 0)],
                is_turn_energy: true,
            },
            is_stack: false,
        };
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            first_attach.clone(),
            Action {
                actor: 0,
                action: SimpleAction::Attach {
                    attachments: vec![(1, EnergyType::Fire, 1)],
                    is_turn_energy: true,
                },
                is_stack: false,
            },
        ];
        
        let chosen = player.decision_fn(&mut rng, &state, actions);
        
        // Should choose first attach action found
        assert_eq!(chosen, first_attach);
    }

    #[test]
    fn test_multiple_attack_actions_chooses_first() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Create multiple attack actions (no attach)
        let first_attack = Action {
            actor: 0,
            action: SimpleAction::Attack(0),
            is_stack: false,
        };
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            first_attack.clone(),
            Action {
                actor: 0,
                action: SimpleAction::Attack(1),
                is_stack: false,
            },
        ];
        
        let chosen = player.decision_fn(&mut rng, &state, actions);
        
        // Should choose first attack action found
        assert_eq!(chosen, first_attack);
    }

    #[test]
    #[should_panic(expected = "There should always be at least one playable action")]
    fn test_panics_with_no_actions() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Empty action list should panic
        let actions = vec![];
        player.decision_fn(&mut rng, &state, actions);
    }

    #[test]
    fn test_ignores_rng_and_state() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        
        // Different RNG seeds
        let mut rng1 = StdRng::seed_from_u64(1);
        let mut rng2 = StdRng::seed_from_u64(9999);
        
        // Different states
        let state1 = State::new(&player.get_deck(), &player.get_deck());
        let mut state2 = State::new(&player.get_deck(), &player.get_deck());
        state2.turn_count = 50;
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::Attach {
                    attachments: vec![(1, EnergyType::Grass, 0)],
                    is_turn_energy: true,
                },
                is_stack: false,
            },
        ];
        
        // Should make same decision regardless of RNG or state
        let choice1 = player.decision_fn(&mut rng1, &state1, actions.clone());
        let choice2 = player.decision_fn(&mut rng2, &state2, actions.clone());
        
        assert_eq!(choice1, choice2);
    }

    #[test]
    fn test_debug_format() {
        let (deck, _) = load_test_decks();
        let player = AttachAttackPlayer { deck };
        
        let debug_str = format!("{:?}", player);
        assert_eq!(debug_str, "AttachAttackPlayer");
    }

    #[test]
    fn test_get_deck() {
        let (deck, _) = load_test_decks();
        let original_deck_size = deck.cards.len();
        let player = AttachAttackPlayer { deck };
        
        let retrieved_deck = player.get_deck();
        assert_eq!(retrieved_deck.cards.len(), original_deck_size);
        
        // Verify deck is cloned, not moved
        let second_retrieval = player.get_deck();
        assert_eq!(second_retrieval.cards.len(), original_deck_size);
    }

    #[test]
    fn test_priority_order() {
        let (deck, _) = load_test_decks();
        let mut player = AttachAttackPlayer { deck };
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Test all actions present - should choose attach
        let all_actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::Attack(0),
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
        
        let chosen = player.decision_fn(&mut rng, &state, all_actions);
        match chosen.action {
            SimpleAction::Attach { .. } => {},
            _ => panic!("Should prioritize Attach when available"),
        }
    }
}
