#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_energy_type_from_str() {
        assert_eq!(EnergyType::from_str("Grass"), Some(EnergyType::Grass));
        assert_eq!(EnergyType::from_str("Fire"), Some(EnergyType::Fire));
        assert_eq!(EnergyType::from_str("Water"), Some(EnergyType::Water));
        assert_eq!(EnergyType::from_str("Lightning"), Some(EnergyType::Lightning));
        assert_eq!(EnergyType::from_str("Psychic"), Some(EnergyType::Psychic));
        assert_eq!(EnergyType::from_str("Fighting"), Some(EnergyType::Fighting));
        assert_eq!(EnergyType::from_str("Darkness"), Some(EnergyType::Darkness));
        assert_eq!(EnergyType::from_str("Metal"), Some(EnergyType::Metal));
        assert_eq!(EnergyType::from_str("Dragon"), Some(EnergyType::Dragon));
        assert_eq!(EnergyType::from_str("Colorless"), Some(EnergyType::Colorless));
        assert_eq!(EnergyType::from_str("Invalid"), None);
    }

    #[test]
    fn test_energy_type_display() {
        assert_eq!(format!("{}", EnergyType::Grass), "Grass");
        assert_eq!(format!("{}", EnergyType::Fire), "Fire");
        assert_eq!(format!("{}", EnergyType::Water), "Water");
        assert_eq!(format!("{}", EnergyType::Lightning), "Lightning");
        assert_eq!(format!("{}", EnergyType::Psychic), "Psychic");
        assert_eq!(format!("{}", EnergyType::Fighting), "Fighting");
        assert_eq!(format!("{}", EnergyType::Darkness), "Darkness");
        assert_eq!(format!("{}", EnergyType::Metal), "Metal");
        assert_eq!(format!("{}", EnergyType::Dragon), "Dragon");
        assert_eq!(format!("{}", EnergyType::Colorless), "Colorless");
    }

    #[test]
    fn test_energy_type_ordering() {
        // Verify that EnergyType implements Ord
        let mut energies = vec![
            EnergyType::Fire,
            EnergyType::Grass,
            EnergyType::Water,
        ];
        energies.sort();
        // Just verify it compiles and doesn't panic
        assert_eq!(energies.len(), 3);
    }

    #[test]
    fn test_pokemon_card_equality() {
        let card1 = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: Some(EnergyType::Fire),
            retreat_cost: vec![EnergyType::Colorless],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut card2 = card1.clone();
        assert_eq!(card1, card2);
        
        // Different name but same ID should still be equal
        card2.name = "Different Name".to_string();
        assert_eq!(card1, card2);
        
        // Different ID should not be equal
        card2.id = "A1 002".to_string();
        assert_ne!(card1, card2);
    }

    #[test]
    fn test_trainer_card_equality() {
        let card1 = TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal 20 damage".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        };
        
        let mut card2 = card1.clone();
        assert_eq!(card1, card2);
        
        // Different name but same ID should still be equal
        card2.name = "Different Name".to_string();
        assert_eq!(card1, card2);
        
        // Different ID should not be equal
        card2.id = "P-A 002".to_string();
        assert_ne!(card1, card2);
    }

    #[test]
    fn test_card_get_id() {
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: Some(EnergyType::Fire),
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert_eq!(pokemon.get_id(), "A1 001");
        assert_eq!(trainer.get_id(), "P-A 001");
    }

    #[test]
    fn test_card_get_name() {
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert_eq!(pokemon.get_name(), "Bulbasaur");
        assert_eq!(trainer.get_name(), "Potion");
    }

    #[test]
    fn test_card_is_ex() {
        let normal_pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let ex_pokemon = Card::Pokemon(PokemonCard {
            id: "A1 004".to_string(),
            name: "Venusaur ex".to_string(),
            stage: 2,
            evolves_from: Some("Ivysaur".to_string()),
            hp: 190,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊◊◊◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert!(!normal_pokemon.is_ex());
        assert!(ex_pokemon.is_ex());
        assert!(!trainer.is_ex());
    }

    #[test]
    fn test_card_is_support() {
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let item_trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        let supporter_trainer = Card::Trainer(TrainerCard {
            id: "A1 223".to_string(),
            numeric_id: 223,
            trainer_card_type: TrainerType::Supporter,
            name: "Giovanni".to_string(),
            effect: "Effect".to_string(),
            rarity: "◊◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        assert!(!pokemon.is_support());
        assert!(!item_trainer.is_support());
        assert!(supporter_trainer.is_support());
    }

    #[test]
    fn test_card_is_basic() {
        let basic_pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let stage1_pokemon = Card::Pokemon(PokemonCard {
            id: "A1 002".to_string(),
            name: "Ivysaur".to_string(),
            stage: 1,
            evolves_from: Some("Bulbasaur".to_string()),
            hp: 90,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert!(basic_pokemon.is_basic());
        assert!(!stage1_pokemon.is_basic());
        assert!(!trainer.is_basic());
    }

    #[test]
    fn test_card_get_type() {
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert_eq!(pokemon.get_type(), Some(EnergyType::Grass));
        assert_eq!(trainer.get_type(), None);
    }

    #[test]
    fn test_card_get_ability() {
        let pokemon_without_ability = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let pokemon_with_ability = Card::Pokemon(PokemonCard {
            id: "A1 007".to_string(),
            name: "Butterfree".to_string(),
            stage: 2,
            evolves_from: Some("Metapod".to_string()),
            hp: 120,
            energy_type: EnergyType::Grass,
            ability: Some(Ability {
                title: "Potent Powder".to_string(),
                effect: "Once during your turn".to_string(),
            }),
            attacks: vec![],
            weakness: Some(EnergyType::Fire),
            retreat_cost: vec![],
            rarity: "◊◊◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        assert!(pokemon_without_ability.get_ability().is_none());
        assert!(pokemon_with_ability.get_ability().is_some());
        assert_eq!(
            pokemon_with_ability.get_ability().unwrap().title,
            "Potent Powder"
        );
    }

    #[test]
    #[should_panic(expected = "Unsupported playable card type")]
    fn test_card_get_attacks_trainer_panics() {
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        trainer.get_attacks();
    }

    #[test]
    fn test_card_get_attacks_pokemon() {
        let attack = Attack {
            energy_required: vec![EnergyType::Grass, EnergyType::Colorless],
            title: "Vine Whip".to_string(),
            fixed_damage: 40,
            effect: None,
        };
        
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![attack.clone()],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let attacks = pokemon.get_attacks();
        assert_eq!(attacks.len(), 1);
        assert_eq!(attacks[0].title, "Vine Whip");
        assert_eq!(attacks[0].fixed_damage, 40);
    }

    #[test]
    fn test_played_card_heal() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 30,
            total_hp: 70,
            attached_energy: vec![],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        // Heal 20
        played_card.heal(20);
        assert_eq!(played_card.remaining_hp, 50);
        
        // Heal beyond max
        played_card.heal(30);
        assert_eq!(played_card.remaining_hp, 70);
    }

    #[test]
    fn test_played_card_apply_damage() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 70,
            total_hp: 70,
            attached_energy: vec![],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        // Apply 20 damage
        played_card.apply_damage(20);
        assert_eq!(played_card.remaining_hp, 50);
        
        // Apply damage beyond remaining HP
        played_card.apply_damage(60);
        assert_eq!(played_card.remaining_hp, 0);
    }

    #[test]
    fn test_played_card_attach_energy() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 70,
            total_hp: 70,
            attached_energy: vec![],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        // Attach 1 Grass energy
        played_card.attach_energy(&EnergyType::Grass, 1);
        assert_eq!(played_card.attached_energy.len(), 1);
        assert_eq!(played_card.attached_energy[0], EnergyType::Grass);
        
        // Attach 2 Fire energy
        played_card.attach_energy(&EnergyType::Fire, 2);
        assert_eq!(played_card.attached_energy.len(), 3);
        assert_eq!(played_card.attached_energy[1], EnergyType::Fire);
        assert_eq!(played_card.attached_energy[2], EnergyType::Fire);
    }

    #[test]
    fn test_played_card_discard_energy() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 70,
            total_hp: 70,
            attached_energy: vec![
                EnergyType::Grass,
                EnergyType::Fire,
                EnergyType::Grass,
            ],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        // Discard one Grass energy
        played_card.discard_energy(&EnergyType::Grass);
        assert_eq!(played_card.attached_energy.len(), 2);
        
        // Should still have one Grass energy
        assert!(played_card.attached_energy.contains(&EnergyType::Grass));
        assert!(played_card.attached_energy.contains(&EnergyType::Fire));
        
        // Try to discard non-existent Water energy
        played_card.discard_energy(&EnergyType::Water);
        assert_eq!(played_card.attached_energy.len(), 2);
    }

    #[test]
    fn test_played_card_is_damaged() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 70,
            total_hp: 70,
            attached_energy: vec![],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        assert!(!played_card.is_damaged());
        
        played_card.remaining_hp = 50;
        assert!(played_card.is_damaged());
    }

    #[test]
    fn test_played_card_has_tool_attached() {
        use crate::tool_ids::ToolId;
        
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let mut played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 70,
            total_hp: 70,
            attached_energy: vec![],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        assert!(!played_card.has_tool_attached());
        
        // This would normally be a real ToolId
        // For testing, we'll just verify the logic works
        played_card.attached_tool = Some(ToolId::A2147GiantCape);
        assert!(played_card.has_tool_attached());
    }

    #[test]
    fn test_card_display() {
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        assert_eq!(format!("{}", pokemon), "Bulbasaur");
        assert_eq!(format!("{}", trainer), "Potion");
    }

    #[test]
    fn test_pokemon_card_debug() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        assert_eq!(format!("{:?}", pokemon_card), "A1 001 Bulbasaur");
        assert_eq!(format!("{:#?}", pokemon_card), "Bulbasaur");
    }

    #[test]
    fn test_trainer_card_debug() {
        let trainer_card = TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        };
        
        assert_eq!(format!("{:?}", trainer_card), "P-A 001 Potion");
        assert_eq!(format!("{:#?}", trainer_card), "Potion");
    }

    #[test]
    fn test_played_card_debug() {
        let pokemon_card = PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        };
        
        let played_card = PlayedCard {
            card: Card::Pokemon(pokemon_card),
            remaining_hp: 50,
            total_hp: 70,
            attached_energy: vec![EnergyType::Grass, EnergyType::Fire],
            attached_tool: None,
            played_this_turn: false,
            ability_used: false,
            poisoned: false,
            paralyzed: false,
            asleep: false,
            cards_behind: vec![],
        };
        
        let debug_str = format!("{:?}", played_card);
        assert!(debug_str.contains("Bulbasaur"));
        assert!(debug_str.contains("50hp"));
        assert!(debug_str.contains("2")); // energy count
        
        let alt_debug_str = format!("{:#?}", played_card);
        assert!(alt_debug_str.contains("Bulbasaur"));
        assert!(alt_debug_str.contains("50hp"));
        assert!(alt_debug_str.contains("[Grass, Fire]")); // energy details
    }

    #[test]
    fn test_status_condition_enum() {
        // Just verify the enum values exist
        let _poisoned = StatusCondition::Poisoned;
        let _paralyzed = StatusCondition::Paralyzed;
        let _asleep = StatusCondition::Asleep;
    }

    #[test]
    fn test_trainer_type_enum() {
        // Verify enum values and equality
        assert_eq!(TrainerType::Supporter, TrainerType::Supporter);
        assert_ne!(TrainerType::Supporter, TrainerType::Item);
        assert_ne!(TrainerType::Item, TrainerType::Tool);
    }

    #[test]
    fn test_card_hash() {
        use std::collections::HashSet;
        
        let pokemon = Card::Pokemon(PokemonCard {
            id: "A1 001".to_string(),
            name: "Bulbasaur".to_string(),
            stage: 0,
            evolves_from: None,
            hp: 70,
            energy_type: EnergyType::Grass,
            ability: None,
            attacks: vec![],
            weakness: None,
            retreat_cost: vec![],
            rarity: "◊".to_string(),
            booster_pack: "Genetic Apex (A1)".to_string(),
        });
        
        let trainer = Card::Trainer(TrainerCard {
            id: "P-A 001".to_string(),
            numeric_id: 1,
            trainer_card_type: TrainerType::Item,
            name: "Potion".to_string(),
            effect: "Heal".to_string(),
            rarity: "◊".to_string(),
            booster_pack: "Promo-A".to_string(),
        });
        
        let mut set = HashSet::new();
        set.insert(pokemon.clone());
        set.insert(trainer.clone());
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&pokemon));
        assert!(set.contains(&trainer));
    }
}