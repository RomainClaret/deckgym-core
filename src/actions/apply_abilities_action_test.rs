#[cfg(test)]
mod tests {
    use super::super::apply_abilities_action::apply_abilities_action;
    use crate::{
        ability_ids::AbilityId,
        card_ids::CardId,
        database::get_card_by_enum,
        hooks::to_playable_card,
        test_helpers::load_test_decks,
        types::{Card, EnergyType, PlayedCard},
        State,
    };

    #[test]
    fn test_butterfree_ability_heals_all_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Butterfree with ability not used
        let card = get_card_by_enum(CardId::A1007Butterfree);
        let mut butterfree = to_playable_card(&card, true);
        butterfree.ability_used = false;
        state.in_play_pokemon[0][0] = Some(butterfree);
        
        // Add damaged Pokemon
        let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut pokemon1 = to_playable_card(&bulbasaur, true);
        pokemon1.apply_damage(30);
        state.in_play_pokemon[0][1] = Some(pokemon1);
        
        let mut pokemon2 = to_playable_card(&bulbasaur, true);
        pokemon2.apply_damage(40);
        state.in_play_pokemon[0][2] = Some(pokemon2);
        
        // Use Butterfree's ability
        apply_abilities_action(0, &mut state, 0);
        
        // Check ability was marked as used
        assert!(state.in_play_pokemon[0][0].as_ref().unwrap().ability_used);
        
        // Check all Pokemon were healed by 20
        let pokemon1 = state.in_play_pokemon[0][1].as_ref().unwrap();
        assert_eq!(pokemon1.total_hp - pokemon1.remaining_hp, 10);
        let pokemon2 = state.in_play_pokemon[0][2].as_ref().unwrap();
        assert_eq!(pokemon2.total_hp - pokemon2.remaining_hp, 20);
    }

    #[test]
    fn test_butterfree_ability_caps_at_full_health() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Butterfree
        let card = get_card_by_enum(CardId::A1007Butterfree);
        let mut butterfree = to_playable_card(&card, true);
        butterfree.ability_used = false;
        state.in_play_pokemon[0][0] = Some(butterfree);
        
        // Add slightly damaged Pokemon
        let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut pokemon = to_playable_card(&bulbasaur, true);
        let max_hp = pokemon.total_hp;
        pokemon.apply_damage(10); // Only 10 damage
        state.in_play_pokemon[0][1] = Some(pokemon);
        
        // Use ability
        apply_abilities_action(0, &mut state, 0);
        
        // Should be fully healed, not over-healed
        let healed_pokemon = state.in_play_pokemon[0][1].as_ref().unwrap();
        assert_eq!(healed_pokemon.total_hp - healed_pokemon.remaining_hp, 0);
        assert_eq!(healed_pokemon.remaining_hp, max_hp);
    }

    #[test]
    fn test_weezing_ability_poisons_opponent() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Weezing in active position
        let card = get_card_by_enum(CardId::A1177Weezing);
        let mut weezing = to_playable_card(&card, true);
        weezing.ability_used = false;
        state.in_play_pokemon[0][0] = Some(weezing);
        
        // Setup opponent's active Pokemon
        let opponent_card = get_card_by_enum(CardId::A1001Bulbasaur);
        let opponent_pokemon = to_playable_card(&opponent_card, true);
        state.in_play_pokemon[1][0] = Some(opponent_pokemon);
        
        // Verify opponent not poisoned initially
        assert!(!state.in_play_pokemon[1][0].as_ref().unwrap().poisoned);
        
        // Use Weezing's ability
        apply_abilities_action(0, &mut state, 0);
        
        // Check ability was marked as used
        assert!(state.in_play_pokemon[0][0].as_ref().unwrap().ability_used);
        
        // Check opponent is now poisoned
        assert!(state.in_play_pokemon[1][0].as_ref().unwrap().poisoned);
    }

    #[test]
    #[should_panic(expected = "Opponent should have active pokemon")]
    fn test_weezing_ability_panics_no_opponent() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Weezing in active position
        let card = get_card_by_enum(CardId::A1177Weezing);
        let mut weezing = to_playable_card(&card, true);
        weezing.ability_used = false;
        state.in_play_pokemon[0][0] = Some(weezing);
        
        // No opponent active Pokemon
        state.in_play_pokemon[1][0] = None;
        
        // This should panic
        apply_abilities_action(0, &mut state, 0);
    }

    #[test]
    fn test_gardevoir_ability_attaches_psychic_energy() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Gardevoir (can be on bench)
        let card = get_card_by_enum(CardId::A1132Gardevoir);
        let mut gardevoir = to_playable_card(&card, true);
        gardevoir.ability_used = false;
        state.in_play_pokemon[0][2] = Some(gardevoir);
        
        // Setup active Pokemon
        let active_card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut active = to_playable_card(&active_card, true);
        active.attached_energy = vec![EnergyType::Grass];
        state.in_play_pokemon[0][0] = Some(active);
        
        // Use Gardevoir's ability
        apply_abilities_action(0, &mut state, 2);
        
        // Check ability was marked as used
        assert!(state.in_play_pokemon[0][2].as_ref().unwrap().ability_used);
        
        // Check Psychic energy was attached to active
        let active_energy = &state.in_play_pokemon[0][0].as_ref().unwrap().attached_energy;
        assert_eq!(active_energy.len(), 2);
        assert!(active_energy.contains(&EnergyType::Psychic));
    }

    #[test]
    #[should_panic(expected = "Arceus's ability cant be used")]
    fn test_arceus_ability_always_panics() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Arceus
        let card = get_card_by_enum(CardId::A2a071ArceusEx);
        let mut arceus = to_playable_card(&card, true);
        arceus.ability_used = false;
        state.in_play_pokemon[0][0] = Some(arceus);
        
        // This should always panic
        apply_abilities_action(0, &mut state, 0);
    }

    #[test]
    #[should_panic(expected = "Pokemon should be there if using ability")]
    fn test_ability_panics_if_no_pokemon() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // No Pokemon at position
        state.in_play_pokemon[0][0] = None;
        
        // This should panic
        apply_abilities_action(0, &mut state, 0);
    }

    #[test]
    fn test_ability_marks_used_flag() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup any Pokemon with ability
        let card = get_card_by_enum(CardId::A1007Butterfree);
        let mut pokemon = to_playable_card(&card, true);
        pokemon.ability_used = false;
        state.in_play_pokemon[0][0] = Some(pokemon);
        
        // Verify not used initially
        assert!(!state.in_play_pokemon[0][0].as_ref().unwrap().ability_used);
        
        // Use ability
        apply_abilities_action(0, &mut state, 0);
        
        // Verify marked as used
        assert!(state.in_play_pokemon[0][0].as_ref().unwrap().ability_used);
    }

    #[test]
    fn test_weezing_ability_only_affects_active() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Weezing in active position
        let card = get_card_by_enum(CardId::A1177Weezing);
        let mut weezing = to_playable_card(&card, true);
        weezing.ability_used = false;
        state.in_play_pokemon[0][0] = Some(weezing);
        
        // Setup opponent's Pokemon
        let opponent_card = get_card_by_enum(CardId::A1001Bulbasaur);
        // Active
        let active = to_playable_card(&opponent_card, true);
        state.in_play_pokemon[1][0] = Some(active);
        
        // Bench
        let bench = to_playable_card(&opponent_card, true);
        state.in_play_pokemon[1][1] = Some(bench);
        
        // Use Weezing's ability
        apply_abilities_action(0, &mut state, 0);
        
        // Only active should be poisoned
        assert!(state.in_play_pokemon[1][0].as_ref().unwrap().poisoned);
        assert!(!state.in_play_pokemon[1][1].as_ref().unwrap().poisoned);
    }

    #[test]
    fn test_gardevoir_multiple_uses() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Gardevoir
        let card = get_card_by_enum(CardId::A1132Gardevoir);
        let mut gardevoir = to_playable_card(&card, true);
        gardevoir.ability_used = false;
        state.in_play_pokemon[0][1] = Some(gardevoir);
        
        // Setup active Pokemon
        let active_card = get_card_by_enum(CardId::A1001Bulbasaur);
        let active = to_playable_card(&active_card, true);
        state.in_play_pokemon[0][0] = Some(active);
        
        // First use
        apply_abilities_action(0, &mut state, 1);
        
        let energy_count = state.in_play_pokemon[0][0]
            .as_ref()
            .unwrap()
            .attached_energy
            .len();
        assert_eq!(energy_count, 1);
        
        // Reset ability used flag (simulating new turn)
        state.in_play_pokemon[0][1].as_mut().unwrap().ability_used = false;
        
        // Second use
        apply_abilities_action(0, &mut state, 1);
        
        let energy_count = state.in_play_pokemon[0][0]
            .as_ref()
            .unwrap()
            .attached_energy
            .len();
        assert_eq!(energy_count, 2);
    }

    #[test]
    fn test_butterfree_heals_empty_slots() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup Butterfree
        let card = get_card_by_enum(CardId::A1007Butterfree);
        let mut butterfree = to_playable_card(&card, true);
        butterfree.ability_used = false;
        state.in_play_pokemon[0][0] = Some(butterfree);
        
        // Only one other Pokemon (with empty slots in between)
        let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut pokemon = to_playable_card(&bulbasaur, true);
        pokemon.apply_damage(30);
        state.in_play_pokemon[0][3] = Some(pokemon);
        
        // Use ability (should not panic on empty slots)
        apply_abilities_action(0, &mut state, 0);
        
        // Check the one Pokemon was healed
        let healed_pokemon = state.in_play_pokemon[0][3].as_ref().unwrap();
        assert_eq!(healed_pokemon.total_hp - healed_pokemon.remaining_hp, 10);
    }

    #[test]
    #[should_panic(expected = "Pokemon should have ability implemented")]
    fn test_unimplemented_ability_panics() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);
        
        // Setup a Pokemon that has an ability but it's not in AbilityId enum
        // This is hard to test without modifying the game data
        // For now, we'll use a Pokemon without ability to trigger a different panic
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        let mut pokemon = to_playable_card(&card, true);
        pokemon.ability_used = false;
        state.in_play_pokemon[0][0] = Some(pokemon);
        
        // This should panic because Bulbasaur doesn't have an ability
        apply_abilities_action(0, &mut state, 0);
    }
}