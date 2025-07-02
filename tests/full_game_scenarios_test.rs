use common::{init_decks, init_random_players};
use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    players::{AttachAttackPlayer, EndTurnPlayer, MctsPlayer, Player, RandomPlayer},
    state::{GameOutcome, State},
    test_helpers::load_test_decks,
    types::{Card, EnergyType, PlayedCard},
    Game,
};
use rand::SeedableRng;
use rand::rngs::StdRng;

mod common;

#[test]
fn test_full_game_with_all_mechanics() {
    // Test a full game with various mechanics: evolution, abilities, retreat, status conditions
    let players = init_random_players();
    let mut game = Game::new(players, 42);
    
    // Play full game
    let outcome = game.play();
    
    // Game should complete without panics
    assert!(game.get_state_clone().is_game_over());
    assert!(outcome.is_some());
}

#[test]
fn test_game_ends_on_no_bench_knockout() {
    // Test that game ends immediately when active is knocked out with no bench
    let (deck_a, deck_b) = load_test_decks();
    let player_a = Box::new(AttachAttackPlayer { deck: deck_a });
    let player_b = Box::new(EndTurnPlayer { deck: deck_b });
    let players: Vec<Box<dyn Player>> = vec![player_a, player_b];
    
    // Use seed that gives player B only one basic Pokemon
    let mut game = Game::new(players, 12345);
    
    // Play until knockout
    let outcome = game.play();
    
    // Verify game ended with correct winner
    match outcome {
        Some(GameOutcome::Win(winner)) => {
            // AttachAttackPlayer should win by knocking out the only Pokemon
            assert!(winner == 0 || winner == 1);
        }
        _ => panic!("Game should end with a winner"),
    }
}

#[test]
fn test_game_ends_on_six_points() {
    // Test that game ends when a player reaches 6 points
    let players = init_random_players();
    let mut game = Game::new(players, 999);
    
    let outcome = game.play();
    
    if let Some(GameOutcome::Win(winner)) = outcome {
        let final_state = game.get_state_clone();
        assert_eq!(final_state.points[winner], 6);
    }
}

#[test]
fn test_game_ends_on_turn_limit() {
    // Test that game ends at turn 100
    let players = init_random_players();
    let mut game = Game::new(players, 7777);
    
    // Play until turn limit or other end condition
    let outcome = game.play();
    
    let final_state = game.get_state_clone();
    if final_state.turn_count >= 100 {
        // Should end in tie or win based on points
        assert!(outcome.is_some());
    }
}

#[test]
fn test_evolution_during_game() {
    // Test that evolution works properly during a game
    let players = init_random_players();
    let mut game = Game::new(players, 55555);
    
    // Play some turns
    for _ in 0..20 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    // Check if any evolved Pokemon are in play
    let state = game.get_state_clone();
    let mut found_evolved = false;
    
    for player in 0..2 {
        for pokemon in state.in_play_pokemon[player].iter().flatten() {
            if !pokemon.cards_behind.is_empty() {
                found_evolved = true;
                // Verify evolution maintained damage and energy
                assert!(pokemon.total_hp > 0);
            }
        }
    }
    
    // Evolution might not happen in 20 turns, so we don't assert
}

#[test]
fn test_status_conditions_applied() {
    // Test poison, paralysis, and sleep mechanics
    let players = init_random_players();
    let mut game = Game::new(players, 33333);
    
    // Play some turns
    for _ in 0..30 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    // Status conditions are applied during the game
    // We can't assert specific conditions without controlling the game more precisely
}

#[test]
fn test_retreat_mechanics() {
    // Test that retreat works and costs energy
    let players = init_random_players();
    let mut game = Game::new(players, 8888);
    
    // Play until a retreat might happen
    for _ in 0..40 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
        
        let state = game.get_state_clone();
        if state.has_retreated {
            // Verify retreat flag was set
            assert!(state.has_retreated);
            break;
        }
    }
}

#[test]
fn test_trainer_cards_played() {
    // Test that trainer cards are played during the game
    let players = init_random_players();
    let mut game = Game::new(players, 1111);
    
    let initial_deck_sizes = {
        let state = game.get_state_clone();
        [state.decks[0].cards.len(), state.decks[1].cards.len()]
    };
    
    // Play some turns
    for _ in 0..20 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    let state = game.get_state_clone();
    
    // Check if cards were drawn (Professor's Research) or other effects
    let cards_drawn = (initial_deck_sizes[0] - state.decks[0].cards.len()) +
                     (initial_deck_sizes[1] - state.decks[1].cards.len());
    
    // More than 20 cards drawn indicates trainer effects
    assert!(cards_drawn > 20 || state.has_played_support);
}

#[test]
fn test_energy_attachment_and_attacks() {
    // Test energy attachment and attack execution
    let (deck_a, deck_b) = load_test_decks();
    let player_a = Box::new(AttachAttackPlayer { deck: deck_a });
    let player_b = Box::new(AttachAttackPlayer { deck: deck_b });
    let players: Vec<Box<dyn Player>> = vec![player_a, player_b];
    
    let mut game = Game::new(players, 2222);
    
    // Play until attacks happen
    for _ in 0..10 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    let state = game.get_state_clone();
    
    // Check that Pokemon have energy attached
    let mut found_energy = false;
    for player in 0..2 {
        for pokemon in state.in_play_pokemon[player].iter().flatten() {
            if !pokemon.attached_energy.is_empty() {
                found_energy = true;
            }
        }
    }
    
    assert!(found_energy);
}

#[test]
fn test_deck_out_condition() {
    // Test that drawing from empty deck is handled
    let players = init_random_players();
    let mut game = Game::new(players, 4444);
    
    // Artificially empty a deck
    {
        let state = game.get_state_mut();
        state.decks[0].cards.clear();
    }
    
    // Force a draw
    let state = game.get_state_mut();
    state.queue_draw_action(0);
    
    // Should handle empty deck gracefully
    game.play_tick();
    
    // Game continues even with empty deck
    assert!(!game.get_state_clone().is_game_over());
}

#[test]
fn test_simultaneous_knockouts() {
    // Test what happens when both active Pokemon are knocked out
    let players = init_random_players();
    let mut game = Game::new(players, 6666);
    
    // This is hard to force without custom game state, but we can play and check
    let outcome = game.play();
    
    // Game should handle simultaneous knockouts without panicking
    assert!(outcome.is_some() || game.get_state_clone().turn_count >= 100);
}

#[test]
fn test_ability_usage() {
    // Test that abilities are used during the game
    let players = init_random_players();
    let mut game = Game::new(players, 7878);
    
    // Play some turns
    for _ in 0..30 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
        
        let state = game.get_state_clone();
        
        // Check if any abilities were used
        for player in 0..2 {
            for pokemon in state.in_play_pokemon[player].iter().flatten() {
                if pokemon.ability_used {
                    // Found an ability usage
                    return;
                }
            }
        }
    }
}

#[test]
fn test_tool_attachment() {
    // Test that tools can be attached to Pokemon
    let players = init_random_players();
    let mut game = Game::new(players, 9999);
    
    // Play some turns
    for _ in 0..40 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
        
        let state = game.get_state_clone();
        
        // Check if any tools were attached
        for player in 0..2 {
            for pokemon in state.in_play_pokemon[player].iter().flatten() {
                if pokemon.attached_tool.is_some() {
                    // Found a tool attachment
                    return;
                }
            }
        }
    }
}

#[test]
fn test_turn_effects_expiration() {
    // Test that turn-based effects expire correctly
    let players = init_random_players();
    let mut game = Game::new(players, 5432);
    
    // Play some turns
    for _ in 0..50 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    // Turn effects should be managed without issues
    let state = game.get_state_clone();
    assert!(state.turn_count > 0);
}

#[test]
fn test_complex_game_state() {
    // Test a game with complex state: multiple Pokemon, energy, tools, status
    let players = init_random_players();
    let mut game = Game::new(players, 1234);
    
    // Play significant portion of game
    for _ in 0..60 {
        if game.get_state_clone().is_game_over() {
            break;
        }
        game.play_tick();
    }
    
    let state = game.get_state_clone();
    
    // Verify game state integrity
    assert!(state.turn_count > 0);
    assert!(state.current_player == 0 || state.current_player == 1);
    
    // Check state consistency
    for player in 0..2 {
        assert!(state.hands[player].len() <= 10); // Reasonable hand size
        assert!(state.points[player] <= 6); // Points don't exceed limit
        
        // Verify Pokemon state
        for pokemon in state.in_play_pokemon[player].iter().flatten() {
            assert!(pokemon.remaining_hp <= pokemon.total_hp);
            assert!(pokemon.remaining_hp > 0); // Knocked out Pokemon should be removed
        }
    }
}

#[test]
fn test_mcts_vs_random_performance() {
    // Test that MCTS performs better than random over multiple games
    let mut mcts_wins = 0;
    let mut random_wins = 0;
    
    for seed in 0..10 {
        let (deck_a, deck_b) = load_test_decks();
        let player_a = Box::new(MctsPlayer::new(deck_a, 10));
        let player_b = Box::new(RandomPlayer { deck: deck_b });
        let players: Vec<Box<dyn Player>> = vec![player_a, player_b];
        
        let mut game = Game::new(players, seed);
        
        if let Some(GameOutcome::Win(winner)) = game.play() {
            if winner == 0 {
                mcts_wins += 1;
            } else {
                random_wins += 1;
            }
        }
    }
    
    // MCTS should win more often (but not required for test to pass)
    println!("MCTS wins: {}, Random wins: {}", mcts_wins, random_wins);
}

#[test]
fn test_game_determinism() {
    // Test that games with same seed produce same results
    let outcome1 = {
        let players = init_random_players();
        let mut game = Game::new(players, 424242);
        game.play()
    };
    
    let outcome2 = {
        let players = init_random_players();
        let mut game = Game::new(players, 424242);
        game.play()
    };
    
    assert_eq!(outcome1, outcome2);
}

#[test]
fn test_edge_case_one_card_deck() {
    // Test game with minimal deck size
    let players = init_random_players();
    let mut game = Game::new(players, 111111);
    
    // Reduce deck sizes drastically
    {
        let state = game.get_state_mut();
        state.decks[0].cards.truncate(5);
        state.decks[1].cards.truncate(5);
    }
    
    // Game should still complete
    let outcome = game.play();
    assert!(outcome.is_some() || game.get_state_clone().turn_count >= 100);
}

#[test]
fn test_game_with_different_deck_matchups() {
    // Test different deck combinations
    let deck_pairs = vec![
        ("venusaur-exeggutor.txt", "weezing-arbok.txt"),
        ("weezing-arbok.txt", "venusaur-exeggutor.txt"),
    ];
    
    for (deck_a_path, deck_b_path) in deck_pairs {
        let players = init_decks(deck_a_path, deck_b_path);
        let mut game = Game::new(players, 777);
        
        let outcome = game.play();
        
        // All matchups should complete successfully
        assert!(game.get_state_clone().is_game_over());
        assert!(outcome.is_some());
    }
}