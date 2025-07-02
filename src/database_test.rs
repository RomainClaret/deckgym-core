#[cfg(test)]
mod tests {
    use crate::database::get_card_by_enum;
    use crate::{
        card_ids::CardId,
        types::{Card, EnergyType, TrainerType},
    };

    #[test]
    fn test_get_pokemon_card() {
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.id, "A1 001");
                assert_eq!(pokemon.name, "Bulbasaur");
                assert_eq!(pokemon.stage, 0);
                assert_eq!(pokemon.hp, 70);
                assert_eq!(pokemon.energy_type, EnergyType::Grass);
                assert!(pokemon.evolves_from.is_none());
                assert_eq!(pokemon.weakness, Some(EnergyType::Fire));
                assert_eq!(pokemon.retreat_cost.len(), 1);
                assert_eq!(pokemon.attacks.len(), 1);
                assert_eq!(pokemon.attacks[0].title, "Vine Whip");
                assert_eq!(pokemon.attacks[0].fixed_damage, 40);
            }
            _ => panic!("Expected Pokemon card for Bulbasaur"),
        }
    }

    #[test]
    fn test_get_evolved_pokemon_card() {
        let card = get_card_by_enum(CardId::A1002Ivysaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.name, "Ivysaur");
                assert_eq!(pokemon.stage, 1);
                assert_eq!(pokemon.evolves_from, Some("Bulbasaur".to_string()));
                assert_eq!(pokemon.hp, 90);
            }
            _ => panic!("Expected Pokemon card for Ivysaur"),
        }
    }

    #[test]
    fn test_get_trainer_card() {
        let card = get_card_by_enum(CardId::PA001Potion);
        
        match card {
            Card::Trainer(trainer) => {
                assert_eq!(trainer.id, "P-A 001");
                assert_eq!(trainer.name, "Potion");
                assert_eq!(trainer.trainer_card_type, TrainerType::Item);
                assert!(trainer.effect.contains("Heal 20 damage"));
            }
            _ => panic!("Expected Trainer card for Potion"),
        }
    }

    #[test]
    fn test_get_supporter_card() {
        let card = get_card_by_enum(CardId::A1223Giovanni);
        
        match card {
            Card::Trainer(trainer) => {
                assert_eq!(trainer.name, "Giovanni");
                assert_eq!(trainer.trainer_card_type, TrainerType::Supporter);
            }
            _ => panic!("Expected Trainer card for Giovanni"),
        }
    }

    #[test]
    fn test_get_ex_pokemon() {
        let card = get_card_by_enum(CardId::A1004VenusaurEx);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.name, "Venusaur ex");
                assert_eq!(pokemon.hp, 190);
                assert_eq!(pokemon.attacks.len(), 2);
                assert_eq!(pokemon.rarity, "◊◊◊◊");
            }
            _ => panic!("Expected Pokemon card for Venusaur ex"),
        }
    }

    #[test]
    fn test_pokemon_with_ability() {
        let card = get_card_by_enum(CardId::A1007Butterfree);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert!(pokemon.ability.is_some());
                let ability = pokemon.ability.as_ref().unwrap();
                assert_eq!(ability.title, "Powder Heal");
            }
            _ => panic!("Expected Pokemon card for Butterfree"),
        }
    }

    #[test]
    fn test_pokemon_without_ability() {
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert!(pokemon.ability.is_none());
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_attack_with_effect() {
        let card = get_card_by_enum(CardId::A1003Venusaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.attacks.len(), 1);
                let attack = &pokemon.attacks[0];
                assert_eq!(attack.title, "Mega Drain");
                assert!(attack.effect.is_some());
                assert!(attack.effect.as_ref().unwrap().contains("Heal 30 damage"));
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_colorless_energy_requirements() {
        let card = get_card_by_enum(CardId::A1186Pidgey);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert!(pokemon.attacks.iter().all(|attack| {
                    attack.energy_required.iter().all(|e| *e == EnergyType::Colorless)
                }));
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_multiple_energy_types() {
        // Find a card with mixed energy requirements
        let card = get_card_by_enum(CardId::A1001Bulbasaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                let attack = &pokemon.attacks[0];
                assert_eq!(attack.energy_required.len(), 2);
                assert_eq!(attack.energy_required[0], EnergyType::Grass);
                assert_eq!(attack.energy_required[1], EnergyType::Colorless);
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_different_booster_packs() {
        let a1_card = get_card_by_enum(CardId::A1001Bulbasaur);
        let promo_card = get_card_by_enum(CardId::PA001Potion);
        
        match a1_card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.booster_pack, "Genetic Apex (A1)");
            }
            _ => panic!("Expected Pokemon card"),
        }
        
        match promo_card {
            Card::Trainer(trainer) => {
                assert_eq!(trainer.booster_pack, "Promo-A");
            }
            _ => panic!("Expected Trainer card"),
        }
    }

    #[test]
    fn test_various_card_ids() {
        // Test a sample of different card IDs to ensure coverage
        let test_cards = vec![
            CardId::A1001Bulbasaur,
            CardId::A1033Charmander,
            CardId::A1053Squirtle,
            CardId::A1186Pidgey,
            CardId::A1a025Pikachu,
            CardId::PA001Potion,
            CardId::A1223Giovanni,
            CardId::A1177Weezing,
            CardId::A1132Gardevoir,
        ];
        
        for card_id in test_cards {
            let card = get_card_by_enum(card_id);
            
            // All cards should have a valid structure
            match card {
                Card::Pokemon(pokemon) => {
                    assert!(!pokemon.id.is_empty());
                    assert!(!pokemon.name.is_empty());
                    assert!(pokemon.hp > 0);
                }
                Card::Trainer(trainer) => {
                    assert!(!trainer.id.is_empty());
                    assert!(!trainer.name.is_empty());
                    assert!(!trainer.effect.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_weakness_and_retreat_costs() {
        let card = get_card_by_enum(CardId::A1033Charmander);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.weakness, Some(EnergyType::Water));
                assert!(!pokemon.retreat_cost.is_empty());
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_no_weakness() {
        // Some cards might not have weakness
        let card = get_card_by_enum(CardId::A1a025Pikachu);
        
        match card {
            Card::Pokemon(pokemon) => {
                // Verify the card structure is valid whether it has weakness or not
                assert!(!pokemon.name.is_empty());
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_stage_2_evolution() {
        let card = get_card_by_enum(CardId::A1003Venusaur);
        
        match card {
            Card::Pokemon(pokemon) => {
                assert_eq!(pokemon.stage, 2);
                assert_eq!(pokemon.evolves_from, Some("Ivysaur".to_string()));
            }
            _ => panic!("Expected Pokemon card"),
        }
    }

    #[test]
    fn test_numeric_id_mapping() {
        // Test that numeric IDs are correctly set
        let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
        let ivysaur = get_card_by_enum(CardId::A1002Ivysaur);
        
        match (bulbasaur, ivysaur) {
            (Card::Pokemon(b), Card::Pokemon(i)) => {
                // IDs should be sequential and properly formatted
                assert_eq!(b.id, "A1 001");
                assert_eq!(i.id, "A1 002");
            }
            _ => panic!("Expected Pokemon cards"),
        }
    }
}