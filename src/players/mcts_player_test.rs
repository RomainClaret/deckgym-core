#[cfg(test)]
mod tests {
    use super::super::mcts_player::MctsPlayer;
    use super::super::Player;
    use crate::{
        actions::{Action, SimpleAction},
        card_ids::CardId,
        database::get_card_by_enum,
        hooks::to_playable_card,
        test_helpers::load_test_decks,
        types::{Card, EnergyType, PlayedCard},
        Deck, State,
    };
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_mcts_player_creation() {
        let (deck, _) = load_test_decks();
        let player = MctsPlayer::new(deck.clone(), 100);
        
        assert_eq!(player.get_deck().cards.len(), deck.cards.len());
        assert_eq!(player.iterations, 100);
    }

    #[test]
    fn test_mcts_player_single_action() {
        let (deck, _) = load_test_decks();
        let mut player = MctsPlayer::new(deck, 10);
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
    fn test_mcts_player_debug_format() {
        let (deck, _) = load_test_decks();
        let player = MctsPlayer::new(deck, 50);
        
        let debug_str = format!("{:?}", player);
        assert_eq!(debug_str, "MctsPlayer with 50 iterations");
    }

    #[test]
    fn test_mcts_player_get_deck() {
        let (deck, _) = load_test_decks();
        let original_deck_size = deck.cards.len();
        let player = MctsPlayer::new(deck, 100);
        
        let retrieved_deck = player.get_deck();
        assert_eq!(retrieved_deck.cards.len(), original_deck_size);
        
        // Verify deck is cloned, not moved
        let second_retrieval = player.get_deck();
        assert_eq!(second_retrieval.cards.len(), original_deck_size);
    }

    #[test]
    fn test_mcts_player_deterministic_with_seed() {
        let (deck, _) = load_test_decks();
        let mut player1 = MctsPlayer::new(deck.clone(), 10);
        let mut player2 = MctsPlayer::new(deck, 10);
        
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
    fn test_mcts_player_different_iterations() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2; // Allow attacks
        
        // Setup a state with meaningful choices
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        if let Some(mut pokemon) = to_playable_card(&card, true) {
            pokemon.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
            state.in_play_pokemon[state.current_player][0] = Some(pokemon);
        }
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::Attack(0),
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
        ];
        
        // Player with more iterations might make different choices
        let mut player_low = MctsPlayer::new(deck_a.clone(), 1);
        let mut player_high = MctsPlayer::new(deck_a, 100);
        
        let mut rng = StdRng::seed_from_u64(42);
        
        // Both should make valid choices
        let choice_low = player_low.decision_fn(&mut rng.clone(), &state, actions.clone());
        let choice_high = player_high.decision_fn(&mut rng, &state, actions.clone());
        
        // Verify both made valid choices
        assert!(actions.contains(&choice_low));
        assert!(actions.contains(&choice_high));
    }

    #[test]
    fn test_mcts_node_lookup_caching() {
        let (deck, _) = load_test_decks();
        let mut player = MctsPlayer::new(deck, 10);
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
        ];
        
        // First call should populate node_lookup
        assert_eq!(player.node_lookup.len(), 0);
        player.decision_fn(&mut rng, &state, actions.clone());
        assert!(player.node_lookup.len() > 0, "Should cache nodes in lookup table");
        
        let cached_nodes = player.node_lookup.len();
        
        // Second call with same state might reuse cached nodes
        player.decision_fn(&mut rng, &state, actions);
        assert!(player.node_lookup.len() >= cached_nodes, "Should maintain or grow cache");
    }

    #[test]
    fn test_mcts_player_handles_winning_position() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Create a near-winning position
        state.points = [2, 0]; // Player 0 almost winning
        state.turn_count = 10;
        
        let mut player = MctsPlayer::new(deck_a, 50);
        let mut rng = StdRng::seed_from_u64(42);
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
        ];
        
        // Should handle terminal/near-terminal states
        let choice = player.decision_fn(&mut rng, &state, actions.clone());
        assert_eq!(choice, actions[0]);
    }

    #[test]
    fn test_mcts_player_multiple_action_types() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup state with various action possibilities
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        if let Some(mut pokemon) = to_playable_card(&card, true) {
            pokemon.attached_energy = vec![EnergyType::Grass];
            state.in_play_pokemon[state.current_player][0] = Some(pokemon);
        }
        state.current_energy = Some(EnergyType::Grass);
        
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
        
        let mut player = MctsPlayer::new(deck_a, 20);
        let mut rng = StdRng::seed_from_u64(42);
        
        // Should explore different action types
        let chosen = player.decision_fn(&mut rng, &state, actions.clone());
        assert!(actions.contains(&chosen), "Should choose a valid action");
    }

    #[test]
    fn test_mcts_player_zero_iterations() {
        let (deck, _) = load_test_decks();
        let mut player = MctsPlayer::new(deck, 0);
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
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
        ];
        
        // With 0 iterations, should still return a valid action
        let choice = player.decision_fn(&mut rng, &state, actions.clone());
        assert!(actions.contains(&choice));
    }

    #[test]
    #[should_panic(expected = "There should be at least one child node")]
    fn test_mcts_player_panics_with_no_actions() {
        let (deck, _) = load_test_decks();
        let mut player = MctsPlayer::new(deck, 10);
        let mut rng = StdRng::seed_from_u64(42);
        let state = State::new(&player.get_deck(), &player.get_deck());
        
        // Empty action list should eventually panic
        let actions = vec![];
        player.decision_fn(&mut rng, &state, actions);
    }

    #[test]
    fn test_mcts_player_respects_actor() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        let mut player = MctsPlayer::new(deck_a, 10);
        let mut rng = StdRng::seed_from_u64(42);
        
        // Actions for different actors
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
            Action {
                actor: 1,
                action: SimpleAction::DrawCard,
                is_stack: false,
            },
        ];
        
        let chosen = player.decision_fn(&mut rng, &state, actions.clone());
        // Should use the first action's actor for investigation
        assert!(actions.contains(&chosen));
    }

    #[test]
    fn test_mcts_player_simulation_convergence() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Create a state where one action is clearly better
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        if let Some(mut pokemon) = to_playable_card(&card, true) {
            pokemon.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
            pokemon.apply_damage(40); // Damaged pokemon
            state.in_play_pokemon[0][0] = Some(pokemon.clone());
            
            // Opponent has low HP pokemon
            pokemon.apply_damage(50);
            state.in_play_pokemon[1][0] = Some(pokemon);
        }
        
        let actions = vec![
            Action {
                actor: 0,
                action: SimpleAction::Attack(0), // Should be preferred
                is_stack: false,
            },
            Action {
                actor: 0,
                action: SimpleAction::EndTurn,
                is_stack: false,
            },
        ];
        
        // With many iterations, should converge to attack
        let mut player = MctsPlayer::new(deck_a, 100);
        let mut rng = StdRng::seed_from_u64(42);
        
        let mut attack_count = 0;
        for _ in 0..10 {
            let choice = player.decision_fn(&mut rng, &state, actions.clone());
            if matches!(choice.action, SimpleAction::Attack(_)) {
                attack_count += 1;
            }
        }
        
        // Should prefer attack most of the time
        assert!(attack_count >= 7, "MCTS should converge to better action");
    }
}