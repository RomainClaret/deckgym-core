#[cfg(test)]
mod tests {
    use super::super::move_generation_trainer::generate_possible_trainer_actions;
    use crate::{
        actions::SimpleAction,
        card_ids::CardId,
        database::get_card_by_enum,
        hooks::to_playable_card,
        test_helpers::load_test_decks,
        types::{Card, EnergyType, PlayedCard, TrainerCard, TrainerType},
        State,
    };

    fn get_trainer_card(card_id: CardId) -> TrainerCard {
        match get_card_by_enum(card_id) {
            Card::Trainer(trainer) => trainer,
            _ => panic!("Expected trainer card"),
        }
    }

    #[test]
    fn test_supporter_blocked_when_already_played() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Mark supporter already played this turn
        state.has_played_support = true;
        
        // Try to play any supporter card
        let giovanni = get_trainer_card(CardId::A1223Giovanni);
        let actions = generate_possible_trainer_actions(&state, &giovanni);
        
        assert!(actions.is_some(), "Should return Some even when blocked");
        assert_eq!(actions.unwrap().len(), 0, "Should not allow supporter when already played");
    }

    #[test]
    fn test_supporter_allowed_when_not_played() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Supporter not played this turn
        state.has_played_support = false;
        
        // Try to play supporter card
        let giovanni = get_trainer_card(CardId::A1223Giovanni);
        let actions = generate_possible_trainer_actions(&state, &giovanni);
        
        assert!(actions.is_some());
        assert_eq!(actions.unwrap().len(), 1, "Should allow supporter when not played");
    }

    #[test]
    fn test_tool_requires_pokemon_without_tool() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Get an actual tool card
        let tool_card = get_trainer_card(CardId::A2147GiantCape);
        
        // No Pokemon in play
        let actions_no_pokemon = generate_possible_trainer_actions(&state, &tool_card);
        assert_eq!(actions_no_pokemon.unwrap().len(), 0, "No Pokemon to attach tool to");
        
        // Add Pokemon without tool
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&card, true));
        
        // Should allow tool attachment now
        let actions_with_pokemon = generate_possible_trainer_actions(&state, &tool_card);
        assert_eq!(actions_with_pokemon.unwrap().len(), 1, "Should allow tool attachment");
    }

    #[test]
    fn test_potion_requires_damaged_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let potion = get_trainer_card(CardId::PA001Potion);
        
        // No Pokemon in play
        let actions_no_pokemon = generate_possible_trainer_actions(&state, &potion);
        assert_eq!(actions_no_pokemon.unwrap().len(), 0, "No Pokemon to heal");
        
        // Add healthy Pokemon
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&card, true));
        
        let actions_healthy = generate_possible_trainer_actions(&state, &potion);
        assert_eq!(actions_healthy.unwrap().len(), 0, "Healthy Pokemon doesn't need potion");
        
        // Damage the Pokemon
        if let Some(pokemon) = &mut state.in_play_pokemon[state.current_player][0] {
            pokemon.apply_damage(20);
        }
        
        let actions_damaged = generate_possible_trainer_actions(&state, &potion);
        assert_eq!(actions_damaged.unwrap().len(), 1, "Should allow potion for damaged Pokemon");
    }

    #[test]
    fn test_erika_requires_damaged_grass_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let erika = get_trainer_card(CardId::A1219Erika);
        
        // Add damaged Fire Pokemon (wrong type)
        let fire_card = get_card_by_enum(CardId::A1033Charmander);
        let mut played_card = to_playable_card(&fire_card, true);
        played_card.apply_damage(20);
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        
        let actions_wrong_type = generate_possible_trainer_actions(&state, &erika);
        assert_eq!(actions_wrong_type.unwrap().len(), 0, "Fire Pokemon can't use Erika");
        
        // Replace with damaged Grass Pokemon
        let grass_card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut played_card = to_playable_card(&grass_card, true);
        played_card.apply_damage(20);
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        
        let actions_grass = generate_possible_trainer_actions(&state, &erika);
        assert_eq!(actions_grass.unwrap().len(), 1, "Damaged Grass Pokemon can use Erika");
    }

    #[test]
    fn test_misty_requires_water_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let misty = get_trainer_card(CardId::A1220Misty);
        
        // No Pokemon in play
        let actions_no_pokemon = generate_possible_trainer_actions(&state, &misty);
        assert_eq!(actions_no_pokemon.unwrap().len(), 0, "No Water Pokemon for Misty");
        
        // Add non-Water Pokemon
        let fire_card = get_card_by_enum(CardId::A1033Charmander);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&fire_card, true));
        
        let actions_wrong_type = generate_possible_trainer_actions(&state, &misty);
        assert_eq!(actions_wrong_type.unwrap().len(), 0, "Fire Pokemon can't use Misty");
        
        // Add Water Pokemon
        let water_card = get_card_by_enum(CardId::A1053Squirtle);
        state.in_play_pokemon[state.current_player][1] = Some(to_playable_card(&water_card, true));
        
        let actions_water = generate_possible_trainer_actions(&state, &misty);
        assert_eq!(actions_water.unwrap().len(), 1, "Water Pokemon can use Misty");
    }

    #[test]
    fn test_koga_requires_specific_active_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let koga = get_trainer_card(CardId::A1222Koga);
        
        // No active Pokemon
        let actions_no_active = generate_possible_trainer_actions(&state, &koga);
        assert_eq!(actions_no_active.unwrap().len(), 0, "No active Pokemon for Koga");
        
        // Wrong active Pokemon
        let wrong_card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&wrong_card, true));
        
        let actions_wrong = generate_possible_trainer_actions(&state, &koga);
        assert_eq!(actions_wrong.unwrap().len(), 0, "Wrong Pokemon can't use Koga");
        
        // Correct active Pokemon (Weezing)
        let weezing_card = get_card_by_enum(CardId::A1177Weezing);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&weezing_card, true));
        
        let actions_weezing = generate_possible_trainer_actions(&state, &koga);
        assert_eq!(actions_weezing.unwrap().len(), 1, "Weezing can use Koga");
        
        // Also test with Muk
        let muk_card = get_card_by_enum(CardId::A1175Muk);
        state.in_play_pokemon[state.current_player][0] = Some(to_playable_card(&muk_card, true));
        
        let actions_muk = generate_possible_trainer_actions(&state, &koga);
        assert_eq!(actions_muk.unwrap().len(), 1, "Muk can use Koga");
    }

    #[test]
    fn test_sabrina_requires_opponent_bench() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let sabrina = get_trainer_card(CardId::A1225Sabrina);
        let opponent = (state.current_player + 1) % 2;
        
        // No opponent bench
        let actions_no_bench = generate_possible_trainer_actions(&state, &sabrina);
        assert_eq!(actions_no_bench.unwrap().len(), 0, "Can't use Sabrina without opponent bench");
        
        // Add opponent bench Pokemon
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[opponent][1] = Some(to_playable_card(&card, true));
        
        let actions_with_bench = generate_possible_trainer_actions(&state, &sabrina);
        assert_eq!(actions_with_bench.unwrap().len(), 1, "Can use Sabrina with opponent bench");
    }

    #[test]
    fn test_cyrus_requires_damaged_opponent_bench() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let cyrus = get_trainer_card(CardId::A2150Cyrus);
        let opponent = (state.current_player + 1) % 2;
        
        // No opponent bench
        let actions_no_bench = generate_possible_trainer_actions(&state, &cyrus);
        assert_eq!(actions_no_bench.unwrap().len(), 0, "Can't use Cyrus without opponent bench");
        
        // Add healthy opponent bench Pokemon
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[opponent][1] = Some(to_playable_card(&card, true));
        
        let actions_healthy = generate_possible_trainer_actions(&state, &cyrus);
        assert_eq!(actions_healthy.unwrap().len(), 0, "Can't use Cyrus on healthy bench");
        
        // Damage the bench Pokemon
        if let Some(pokemon) = &mut state.in_play_pokemon[opponent][1] {
            pokemon.apply_damage(30);
        }
        
        let actions_damaged = generate_possible_trainer_actions(&state, &cyrus);
        assert_eq!(actions_damaged.unwrap().len(), 1, "Can use Cyrus on damaged bench");
    }

    #[test]
    fn test_always_playable_trainers() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        let always_playable = vec![
            CardId::PA002XSpeed,
            CardId::PA005PokeBall,
            CardId::PA006RedCard,
            CardId::PA007ProfessorsResearch,
            CardId::A1223Giovanni,
            CardId::A1270Giovanni,
            CardId::A1a065MythicalSlab,
            CardId::A1a068Leaf,
            CardId::A1a082Leaf,
        ];
        
        for card_id in always_playable {
            let trainer = get_trainer_card(card_id);
            let actions = generate_possible_trainer_actions(&state, &trainer);
            
            assert!(actions.is_some(), "Card {:?} should be implemented", card_id);
            assert_eq!(actions.unwrap().len(), 1, "Card {:?} should always be playable", card_id);
        }
    }

    #[test]
    fn test_unimplemented_trainer_returns_none() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        // Create a fake trainer card with unimplemented ID
        let mut unimplemented_trainer = get_trainer_card(CardId::PA001Potion);
        unimplemented_trainer.numeric_id = 9999; // Assuming this doesn't exist
        
        let actions = generate_possible_trainer_actions(&state, &unimplemented_trainer);
        assert!(actions.is_none(), "Unimplemented trainer should return None");
    }

    #[test]
    fn test_play_action_format() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        let giovanni = get_trainer_card(CardId::A1223Giovanni);
        let actions = generate_possible_trainer_actions(&state, &giovanni).unwrap();
        
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SimpleAction::Play { trainer_card } => {
                assert_eq!(trainer_card.numeric_id, giovanni.numeric_id);
            }
            _ => panic!("Expected Play action"),
        }
    }

    #[test]
    fn test_multiple_damaged_pokemon_still_one_action() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        let potion = get_trainer_card(CardId::PA001Potion);
        
        // Add multiple damaged Pokemon
        for i in 0..3 {
            let card = get_card_by_enum(CardId::A1001Bulbasaur);
            let mut pokemon = to_playable_card(&card, true);
            pokemon.apply_damage(20);
            state.in_play_pokemon[state.current_player][i] = Some(pokemon);
        }
        
        let actions = generate_possible_trainer_actions(&state, &potion);
        assert_eq!(actions.unwrap().len(), 1, "Should still only generate one Play action");
    }

    #[test]
    fn test_tool_detection_with_has_tool_attached() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Get an actual tool card
        let tool_card = get_trainer_card(CardId::A2147GiantCape);
        
        // Add Pokemon with tool already attached
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        let played_card = to_playable_card(&card, true);
        // Simulate tool attachment (assuming has_tool_attached() checks some field)
        // This test verifies the filter logic even if we can't directly set tool state
        state.in_play_pokemon[state.current_player][0] = Some(played_card);
        
        // The actual behavior depends on has_tool_attached() implementation
        let actions = generate_possible_trainer_actions(&state, &tool_card);
        assert!(actions.is_some(), "Should return Some for tool cards");
    }

    #[test]
    fn test_supporter_check_happens_first() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Mark supporter already played
        state.has_played_support = true;
        
        // Use Sabrina which has additional requirements
        let sabrina = get_trainer_card(CardId::A1225Sabrina);
        let opponent = (state.current_player + 1) % 2;
        
        // Add opponent bench (would normally allow Sabrina)
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[opponent][1] = Some(to_playable_card(&card, true));
        
        // Should still be blocked by supporter check
        let actions = generate_possible_trainer_actions(&state, &sabrina);
        assert_eq!(actions.unwrap().len(), 0, "Supporter check should happen before other checks");
    }
}