use crate::{
    actions::SimpleAction,
    card_ids::CardId,
    database::get_card_by_enum,
    hooks::can_play_support,
    types::{EnergyType, TrainerCard, TrainerType},
    State,
};

/// Generate possible actions for a trainer card.
///
/// Returns None instead of panicing if the trainer card is not implemented; this is so that the
/// WASM module can do "feature detection", and know if a card is implemented.
pub fn generate_possible_trainer_actions(
    state: &State,
    trainer_card: &TrainerCard,
) -> Option<Vec<SimpleAction>> {
    if trainer_card.trainer_card_type == TrainerType::Supporter && !can_play_support(state) {
        return Some(vec![]); // dont even check which type it is
    }

    // Pokemon tools can be played if there is a space in the mat for them.
    if trainer_card.trainer_card_type == TrainerType::Tool {
        let in_play_without_tools = state
            .enumerate_in_play_pokemon(state.current_player)
            .filter(|(_, x)| !x.has_tool_attached())
            .count();
        if in_play_without_tools > 0 {
            return Some(vec![SimpleAction::Play {
                trainer_card: trainer_card.clone(),
            }]);
        } else {
            return Some(vec![]);
        }
    }

    let trainer_id = match CardId::from_numeric_id(trainer_card.numeric_id) {
        Some(id) => id,
        None => return None, // Unimplemented trainer card
    };
    match trainer_id {
        CardId::PA001Potion => {
            // There must be at least 1 damaged pokemon in play
            let damaged_count = state
                .enumerate_in_play_pokemon(state.current_player)
                .filter(|(_, x)| x.is_damaged())
                .count();
            if damaged_count > 0 {
                Some(vec![SimpleAction::Play {
                    trainer_card: trainer_card.clone(),
                }])
            } else {
                Some(vec![])
            }
        }
        CardId::A1219Erika | CardId::A1266Erika => {
            // There must be at least 1 damaged Leaf pokemon in play
            let damaged_grass_count = state
                .enumerate_in_play_pokemon(state.current_player)
                .filter(|(_, x)| x.is_damaged() && x.get_energy_type() == Some(EnergyType::Grass))
                .count();
            if damaged_grass_count > 0 {
                Some(vec![SimpleAction::Play {
                    trainer_card: trainer_card.clone(),
                }])
            } else {
                Some(vec![])
            }
        }
        CardId::A1220Misty | CardId::A1267Misty => {
            // There must be at least 1 water pokemon in play
            let water_in_player_count =
                state.num_in_play_of_type(state.current_player, EnergyType::Water);
            if water_in_player_count > 0 {
                Some(vec![SimpleAction::Play {
                    trainer_card: trainer_card.clone(),
                }])
            } else {
                Some(vec![])
            }
        }
        CardId::A1222Koga | CardId::A1269Koga => {
            // If Koga, confirm that Active pokemon is Weezing or Muk
            let active_pokemon = &state.in_play_pokemon[state.current_player][0];
            if let Some(played_card) = active_pokemon {
                let kogable_cards = vec![
                    get_card_by_enum(CardId::A1177Weezing),
                    get_card_by_enum(CardId::A1243Weezing),
                    get_card_by_enum(CardId::A1175Muk),
                ];
                if kogable_cards.contains(&played_card.card) {
                    return Some(vec![SimpleAction::Play {
                        trainer_card: trainer_card.clone(),
                    }]);
                }
            }
            Some(vec![])
        }
        CardId::A1225Sabrina | CardId::A1272Sabrina => {
            // If Sabrina, confirm that opponent has benched pokemon
            let opponent = (state.current_player + 1) % 2;
            let opponent_has_bench = state.enumerate_bench_pokemon(opponent).count() > 0;
            if opponent_has_bench {
                Some(vec![SimpleAction::Play {
                    trainer_card: trainer_card.clone(),
                }])
            } else {
                Some(vec![])
            }
        }
        CardId::A2150Cyrus | CardId::A2190Cyrus => {
            // Enemy has to have at least 1 damaged bench pokemon
            let opponent = (state.current_player + 1) % 2;
            let damaged_bench_count = state
                .enumerate_bench_pokemon(opponent)
                .filter(|(_, x)| x.is_damaged())
                .count();
            if damaged_bench_count > 0 {
                Some(vec![SimpleAction::Play {
                    trainer_card: trainer_card.clone(),
                }])
            } else {
                Some(vec![])
            }
        }
        // These can always be played (support check already done)
        CardId::PA002XSpeed
        | CardId::PA005PokeBall
        | CardId::PA006RedCard
        | CardId::PA007ProfessorsResearch
        | CardId::A1223Giovanni
        | CardId::A1270Giovanni
        | CardId::A1a065MythicalSlab
        | CardId::A1a068Leaf
        | CardId::A1a082Leaf => Some(vec![SimpleAction::Play {
            trainer_card: trainer_card.clone(),
        }]),
        _ => None,
    }
}
