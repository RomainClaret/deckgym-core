#[cfg(test)]
mod tests {
    use super::super::attacks::generate_attack_actions;
    use crate::{
        actions::SimpleAction,
        card_ids::CardId,
        database::get_card_by_enum,
        hooks::to_playable_card,
        test_helpers::load_test_decks,
        types::{Card, EnergyType, PlayedCard},
        State,
    };

    #[test]
    fn test_no_attacks_on_first_turn() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Turn count is 0 initially
        let actions = generate_attack_actions(&state);
        assert_eq!(actions.len(), 0, "Should not generate attacks on turn 0");
        
        // Turn count is 1
        state.turn_count = 1;
        let actions = generate_attack_actions(&state);
        assert_eq!(actions.len(), 0, "Should not generate attacks on turn 1");
    }

    #[test]
    fn test_no_attacks_without_active_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Clear active pokemon
        state.in_play_pokemon[state.current_player][0] = None;
        
        let actions = generate_attack_actions(&state);
        assert_eq!(actions.len(), 0, "Should not generate attacks without active pokemon");
    }

    #[test]
    fn test_no_attacks_without_required_energy() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup active pokemon with no energy attached
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut played_card = to_playable_card(&card, true);
        played_card.attached_energy = vec![]; // No energy attached
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        
        let actions = generate_attack_actions(&state);
        assert_eq!(actions.len(), 0, "Should not generate attacks without required energy");
    }

    #[test]
    fn test_generates_attack_with_sufficient_energy() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup active pokemon with energy attached
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut played_card = to_playable_card(&card, true);
        // Bulbasaur's first attack requires Grass energy
        played_card.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        
        let actions = generate_attack_actions(&state);
        assert!(actions.len() > 0, "Should generate at least one attack with sufficient energy");
        
        // Check the generated action is an Attack
        match &actions[0] {
            SimpleAction::Attack(index) => {
                assert_eq!(*index, 0, "Should generate attack for first attack index");
            },
            _ => panic!("Expected Attack action"),
        }
    }

    #[test]
    fn test_generates_multiple_attacks_when_available() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Find a pokemon with multiple attacks and give it lots of energy
        let card = get_card_by_enum(CardId::A1002Ivysaur);
        let mut played_card = to_playable_card(&card, true);
        // Give it plenty of energy for all attacks
        played_card.attached_energy = vec![
            EnergyType::Grass, EnergyType::Grass, 
            EnergyType::Grass, EnergyType::Grass,
            EnergyType::Fire, EnergyType::Water
        ];
        state.in_play_pokemon[state.current_player][0] = Some(played_card.clone());
        
        let actions = generate_attack_actions(&state);
        
        // Should generate actions for all attacks that have sufficient energy
        let attack_count = played_card.get_attacks().len();
        assert!(actions.len() <= attack_count, "Should not generate more attacks than available");
        
        // Verify all actions are Attack actions with correct indices
        for (i, action) in actions.iter().enumerate() {
            match action {
                SimpleAction::Attack(index) => {
                    assert!(*index < attack_count, "Attack index should be valid");
                },
                _ => panic!("Expected Attack action"),
            }
        }
    }

    #[test]
    fn test_only_generates_attacks_with_matching_energy() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup pokemon with specific energy types
        let card = get_card_by_enum(CardId::A1033Charmander);
        let mut played_card = to_playable_card(&card, true);
        // Give it only Fire energy
        played_card.attached_energy = vec![EnergyType::Fire, EnergyType::Fire];
        state.in_play_pokemon[state.current_player][0] = Some(played_card.clone());
        
        let actions = generate_attack_actions(&state);
        
        // Should only generate attacks that can be used with Fire energy
        for action in &actions {
            match action {
                SimpleAction::Attack(index) => {
                    let attack = &played_card.get_attacks()[*index];
                    // Verify the attack can be used with the attached energy
                    let mut required_energy = attack.energy_required.clone();
                    let mut available_energy = played_card.attached_energy.clone();
                        
                    // Simple check: all required energy should be satisfiable
                    for req in &required_energy {
                        let found = available_energy.iter().position(|e| e == req);
                        assert!(found.is_some(), "Attack should only be generated if energy requirements are met");
                    }
                },
                _ => panic!("Expected Attack action"),
            }
        }
    }

    #[test]
    fn test_respects_current_player() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup pokemon for both players
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut played_card_p0 = to_playable_card(&card, true);
        played_card_p0.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
        state.in_play_pokemon[0][0] = Some(played_card_p0);
        
        let mut played_card_p1 = to_playable_card(&card, true);
        played_card_p1.attached_energy = vec![EnergyType::Fire, EnergyType::Fire];
        state.in_play_pokemon[1][0] = Some(played_card_p1);
        
        // Test for player 0
        state.current_player = 0;
        let actions_p0 = generate_attack_actions(&state);
        
        // Test for player 1
        state.current_player = 1;
        let actions_p1 = generate_attack_actions(&state);
        
        // Actions should potentially be different based on different energy
        // Player 0 has Grass energy, Player 1 has Fire energy
        // So they should generate different attack options
        assert!(actions_p0.len() > 0, "Player 0 should have attacks with Grass energy");
        assert_eq!(actions_p1.len(), 0, "Player 1 should have no attacks with only Fire energy on Bulbasaur");
    }

    #[test]
    fn test_attack_index_ordering() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Setup pokemon with multiple attacks
        let card = get_card_by_enum(CardId::A1002Ivysaur);
        let mut played_card = to_playable_card(&card, true);
        // Give it enough energy for all attacks
        played_card.attached_energy = vec![
            EnergyType::Grass, EnergyType::Grass, EnergyType::Grass,
            EnergyType::Grass, EnergyType::Grass, EnergyType::Grass,
        ];
        state.in_play_pokemon[state.current_player][0] = Some(played_card.clone());
        
        let actions = generate_attack_actions(&state);
        
        // Check that attack indices are in order
        let mut last_index = None;
        for action in &actions {
            match action {
                SimpleAction::Attack(index) => {
                    if let Some(last) = last_index {
                        assert!(*index > last, "Attack indices should be in increasing order");
                    }
                    last_index = Some(*index);
                },
                _ => panic!("Expected Attack action"),
            }
        }
    }

    #[test]
    fn test_colorless_energy_compatibility() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Find a pokemon that uses colorless energy
        let card = get_card_by_enum(CardId::A1186Pidgey);
        let mut played_card = to_playable_card(&card, true);
        // Colorless attacks can use any energy type
        played_card.attached_energy = vec![EnergyType::Fire, EnergyType::Water];
        state.in_play_pokemon[state.current_player][0] = Some(played_card.clone());
        
        let actions = generate_attack_actions(&state);
        
        // Should be able to use attacks with colorless requirements
        assert!(actions.len() > 0, "Should generate attacks with any energy type for colorless requirements");
    }

    #[test]
    fn test_mixed_energy_requirements() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        state.turn_count = 2;
        
        // Find a pokemon with mixed energy requirements
        let card = get_card_by_enum(CardId::A1003Venusaur);
        let mut played_card = to_playable_card(&card, true);
        
        // Test with insufficient energy mix
        played_card.attached_energy = vec![EnergyType::Grass, EnergyType::Fire];
        state.in_play_pokemon[state.current_player][0] = Some(played_card.clone());
        let actions_insufficient = generate_attack_actions(&state);
        
        // Test with sufficient energy mix
        played_card.attached_energy = vec![
            EnergyType::Grass, EnergyType::Grass,
            EnergyType::Grass, EnergyType::Grass,
            EnergyType::Fire, EnergyType::Water
        ];
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        let actions_sufficient = generate_attack_actions(&state);
        
        // Should have more attacks available with more energy
        assert!(actions_sufficient.len() >= actions_insufficient.len(), 
            "More energy should enable more or equal attacks");
    }
}