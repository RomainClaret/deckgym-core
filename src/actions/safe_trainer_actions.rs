use log::debug;
use rand::rngs::StdRng;

use crate::{
    card_ids::CardId,
    types::{Card, TrainerCard},
    State,
};

use super::{
    apply_action_helpers::{apply_common_mutation, Mutations, Probabilities},
    Action, SimpleAction,
};

/// Information-safe version of trainer action forecasting.
/// 
/// This module fixes the information leakage issue where bots could see
/// the exact contents of decks when forecasting trainer card effects.
/// Instead of revealing exact cards, we use probability distributions.
pub fn forecast_trainer_action_safe(
    acting_player: usize,
    state: &State,
    trainer_card: &TrainerCard,
) -> (Probabilities, Mutations) {
    let trainer_id = CardId::from_numeric_id(trainer_card.numeric_id)
        .expect("CardId should be known");
        
    match trainer_id {
        // Deterministic effects (no information leakage)
        CardId::PA001Potion => deterministic_safe(potion_effect_safe),
        CardId::PA002XSpeed => deterministic_safe(turn_effect_safe),
        CardId::A1219Erika | CardId::A1266Erika => deterministic_safe(erika_effect_safe),
        CardId::A1222Koga | CardId::A1269Koga => deterministic_safe(koga_effect_safe),
        CardId::A1223Giovanni | CardId::A1270Giovanni => deterministic_safe(giovanni_effect_safe),
        CardId::A1225Sabrina | CardId::A1272Sabrina => deterministic_safe(sabrina_effect_safe),
        CardId::A1a068Leaf | CardId::A1a082Leaf => deterministic_safe(turn_effect_safe),
        CardId::A2150Cyrus | CardId::A2190Cyrus => deterministic_safe(cyrus_effect_safe),
        CardId::A2147GiantCape => deterministic_safe(attach_tool_safe),
        
        // Probabilistic effects (fixed to not leak information)
        CardId::PA005PokeBall => pokeball_outcomes_safe(acting_player, state),
        CardId::PA006RedCard => red_card_outcomes_safe(acting_player, state),
        CardId::PA007ProfessorsResearch => professor_research_outcomes_safe(acting_player, state),
        CardId::A1220Misty | CardId::A1267Misty => misty_outcomes_safe(),
        CardId::A1a065MythicalSlab => mythical_slab_outcomes_safe(acting_player, state),
        
        _ => panic!("Unsupported Trainer Card: {:?}", trainer_id),
    }
}

fn deterministic_safe(
    mutation: fn(&mut StdRng, &mut State, &Action)
) -> (Probabilities, Mutations) {
    (
        vec![1.0],
        vec![Box::new(move |rng, state, action| {
            apply_common_mutation(state, action);
            mutation(rng, state, action);
        })],
    )
}

/// Professor's Research - Draw 2 cards
/// FIXED: No longer reveals what cards will be drawn
fn professor_research_outcomes_safe(
    acting_player: usize,
    state: &State,
) -> (Probabilities, Mutations) {
    // We represent this as a single outcome with probability 1.0
    // The actual cards drawn are determined when the action is applied
    (
        vec![1.0],
        vec![Box::new(move |_, state, action| {
            apply_common_mutation(state, action);
            
            // Queue draw actions without revealing what will be drawn
            for _ in 0..2 {
                state.queue_draw_action(action.actor);
            }
        })],
    )
}

/// Pokeball - Put 1 random Basic Pokemon from deck into hand
/// FIXED: No longer reveals which basic Pokemon are in the deck
fn pokeball_outcomes_safe(acting_player: usize, state: &State) -> (Probabilities, Mutations) {
    // Check if there are basic Pokemon without revealing which ones
    let has_basic = state.decks[acting_player]
        .cards
        .iter()
        .any(|x| x.is_basic());
        
    if !has_basic {
        // No basics - just shuffle
        deterministic_safe(|rng, state, action| {
            state.decks[action.actor].shuffle(false, rng);
        })
    } else {
        // We know there's at least one basic, but we don't reveal which
        // This is represented as a single outcome
        (
            vec![1.0],
            vec![Box::new(move |rng, state, action| {
                apply_common_mutation(state, action);
                
                // Find all basics (hidden from forecast)
                let basics: Vec<_> = state.decks[action.actor]
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| card.is_basic())
                    .map(|(idx, card)| (idx, card.clone()))
                    .collect();
                
                if !basics.is_empty() {
                    // Randomly select one
                    use rand::seq::SliceRandom;
                    let (idx, card) = basics.choose(rng).unwrap();
                    
                    debug!("Pokeball selected card: {:?}", card);
                    
                    // Add to hand and remove from deck
                    state.hands[action.actor].push(card.clone());
                    state.decks[action.actor].cards.remove(*idx);
                }
                
                state.decks[action.actor].shuffle(false, rng);
            })],
        )
    }
}

/// Red Card - Opponent shuffles hand into deck and draws 3
/// FIXED: No longer reveals opponent's deck contents
fn red_card_outcomes_safe(acting_player: usize, state: &State) -> (Probabilities, Mutations) {
    (
        vec![1.0],
        vec![Box::new(move |rng, state, action| {
            apply_common_mutation(state, action);
            
            let opponent = (action.actor + 1) % 2;
            
            // Shuffle hand into deck
            let opponent_hand = &mut state.hands[opponent];
            let opponent_deck = &mut state.decks[opponent];
            opponent_deck.cards.append(opponent_hand);
            opponent_deck.shuffle(false, rng);
            
            // Queue draw actions without revealing what will be drawn
            for _ in 0..3 {
                state.queue_draw_action(opponent);
            }
        })],
    )
}

/// Mythical Slab - Look at top card, put in hand if Psychic, else bottom
/// FIXED: No longer reveals the top card
fn mythical_slab_outcomes_safe(acting_player: usize, state: &State) -> (Probabilities, Mutations) {
    // We can't know what the top card is during forecast
    // Represent as single outcome that will be resolved at execution
    (
        vec![1.0],
        vec![Box::new(move |_, state, action| {
            apply_common_mutation(state, action);
            
            if let Some(card) = state.decks[action.actor].cards.first() {
                if card.is_psychic_type() {
                    // Put in hand
                    let card = state.decks[action.actor].cards.remove(0);
                    state.hands[action.actor].push(card);
                } else {
                    // Put on bottom
                    let card = state.decks[action.actor].cards.remove(0);
                    state.decks[action.actor].cards.push(card);
                }
            }
        })],
    )
}

/// Misty - Flip coins, attach Water energy for each heads
/// This one is already probabilistic and doesn't leak information
fn misty_outcomes_safe() -> (Probabilities, Mutations) {
    use crate::types::EnergyType;
    
    // 50% no energy, 25% 1 energy, 12.5% 2 energy, etc.
    let probabilities = vec![0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625];
    let mut outcomes: Mutations = vec![];
    
    for j in 0..6 {
        outcomes.push(Box::new(move |_, state, action| {
            apply_common_mutation(state, action);
            
            // Queue energy attachment decisions
            let possible_moves = state
                .enumerate_in_play_pokemon(action.actor)
                .filter(|(_, x)| x.get_energy_type() == Some(EnergyType::Water))
                .map(|(i, _)| SimpleAction::Attach {
                    attachments: vec![(j, EnergyType::Water, i)],
                    is_turn_energy: false,
                })
                .collect::<Vec<_>>();
                
            if !possible_moves.is_empty() {
                state.move_generation_stack
                    .push((action.actor, possible_moves));
            }
        }));
    }
    
    (probabilities, outcomes)
}

// Deterministic effect implementations
fn potion_effect_safe(rng: &mut StdRng, state: &mut State, action: &Action) {
    inner_healing_effect_safe(rng, state, action, 20, None);
}

fn erika_effect_safe(rng: &mut StdRng, state: &mut State, action: &Action) {
    use crate::types::EnergyType;
    inner_healing_effect_safe(rng, state, action, 50, Some(EnergyType::Grass));
}

fn inner_healing_effect_safe(
    _: &mut StdRng,
    state: &mut State,
    action: &Action,
    amount: u32,
    energy: Option<crate::types::EnergyType>,
) {
    use crate::types::EnergyType;
    
    let possible_moves = state
        .enumerate_in_play_pokemon(action.actor)
        .filter(|(_, x)| energy.is_none() || x.get_energy_type() == energy)
        .map(|(i, _)| SimpleAction::Heal {
            in_play_idx: i,
            amount,
        })
        .collect::<Vec<_>>();
        
    if !possible_moves.is_empty() {
        state.move_generation_stack
            .push((action.actor, possible_moves));
    }
}

fn koga_effect_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    // Implementation remains the same as it doesn't leak information
    let possible_moves = state
        .enumerate_in_play_pokemon(action.actor)
        .filter(|(_, x)| x.is_poisoned())
        .map(|(i, _)| SimpleAction::Activate { in_play_idx: i })
        .collect::<Vec<_>>();
        
    if !possible_moves.is_empty() {
        state.move_generation_stack
            .push((action.actor, possible_moves));
    }
}

fn giovanni_effect_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    if let SimpleAction::Play { trainer_card } = &action.action {
        let card = Card::Trainer(trainer_card.clone());
        state.add_turn_effect(card, 0);
    }
}

fn sabrina_effect_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    let opponent_player = (action.actor + 1) % 2;
    let possible_moves = state
        .enumerate_bench_pokemon(opponent_player)
        .map(|(i, _)| SimpleAction::Activate { in_play_idx: i })
        .collect::<Vec<_>>();
        
    state.move_generation_stack
        .push((opponent_player, possible_moves));
}

fn cyrus_effect_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    let opponent_player = (action.actor + 1) % 2;
    let possible_moves = state
        .enumerate_bench_pokemon(opponent_player)
        .filter(|(_, x)| x.is_damaged())
        .map(|(in_play_idx, _)| SimpleAction::Activate { in_play_idx })
        .collect::<Vec<_>>();
        
    state.move_generation_stack
        .push((opponent_player, possible_moves));
}

fn turn_effect_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    if let SimpleAction::Play { trainer_card } = &action.action {
        let card = Card::Trainer(trainer_card.clone());
        state.add_turn_effect(card, 0);
    }
}

fn attach_tool_safe(_: &mut StdRng, state: &mut State, action: &Action) {
    use crate::tool_ids::ToolId;
    
    if let SimpleAction::Play { trainer_card } = &action.action {
        let &tool_id = ToolId::from_trainer_card(trainer_card)
            .expect("ToolId should exist");
            
        let choices = state
            .enumerate_in_play_pokemon(action.actor)
            .filter(|(_, x)| !x.has_tool_attached())
            .map(|(i, _)| SimpleAction::AttachTool {
                in_play_idx: i,
                tool_id,
            })
            .collect::<Vec<_>>();
            
        if !choices.is_empty() {
            state.move_generation_stack
                .push((action.actor, choices));
        }
    }
}

// Helper trait for Card to check Psychic type
trait PsychicCheck {
    fn is_psychic_type(&self) -> bool;
}

impl PsychicCheck for Card {
    fn is_psychic_type(&self) -> bool {
        use crate::types::EnergyType;
        
        match self {
            Card::Pokemon(p) => p.energy_type == EnergyType::Psychic,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::load_test_decks;
    
    #[test]
    fn test_no_information_leakage() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        // Create a Professor's Research card
        let prof_research = TrainerCard {
            id: CardId::PA007ProfessorsResearch as u16,
            name: "Professor's Research".to_string(),
            trainer_type: crate::types::TrainerType::Supporter,
        };
        
        // Forecast should not reveal what cards will be drawn
        let (probs, mutations) = forecast_trainer_action_safe(0, &state, &prof_research);
        
        // Should be a single outcome with probability 1.0
        assert_eq!(probs.len(), 1);
        assert_eq!(probs[0], 1.0);
        assert_eq!(mutations.len(), 1);
    }
    
    #[test]
    fn test_pokeball_no_deck_reveal() {
        let (deck_a, deck_b) = load_test_decks();
        let state = State::new(&deck_a, &deck_b);
        
        let pokeball = TrainerCard {
            id: CardId::PA005PokeBall as u16,
            name: "Poke Ball".to_string(),
            trainer_type: crate::types::TrainerType::Item,
        };
        
        // Should not reveal which basic Pokemon are available
        let (probs, mutations) = forecast_trainer_action_safe(0, &state, &pokeball);
        
        // Should be single outcome (success/fail hidden)
        assert_eq!(probs.len(), 1);
        assert_eq!(mutations.len(), 1);
    }
}