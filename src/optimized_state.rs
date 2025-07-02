use log::{debug, trace};
use rand::{seq::SliceRandom, Rng};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::{
    actions::SimpleAction,
    deck::Deck,
    state::GameOutcome,
    types::{Card, EnergyType, PlayedCard},
};

/// Optimized State struct using Arc for expensive fields to enable cheap cloning.
/// This dramatically improves performance for AI algorithms like MCTS that need
/// to clone state frequently.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct OptimizedState {
    // Turn State - cheap to clone
    pub winner: Option<GameOutcome>,
    pub points: [u8; 2],
    pub turn_count: u8,
    pub current_player: usize,
    
    // Expensive fields wrapped in Arc for copy-on-write
    pub move_generation_stack: Arc<Vec<(usize, Vec<SimpleAction>)>>,
    
    // Core state
    pub(crate) current_energy: Option<EnergyType>,
    pub hands: [Arc<Vec<Card>>; 2],
    pub decks: [Arc<Deck>; 2],
    pub discard_piles: [Arc<Vec<Card>>; 2],
    
    // This is relatively small, so we keep it as-is
    pub in_play_pokemon: [[Option<PlayedCard>; 4]; 2],
    
    // Turn Flags - cheap to clone
    pub(crate) has_played_support: bool,
    pub(crate) has_retreated: bool,
    
    // Expensive field wrapped in Arc
    turn_effects: Arc<BTreeMap<u8, Vec<Card>>>,
}

impl OptimizedState {
    /// Create a new state from decks
    pub(crate) fn new(deck_a: &Deck, deck_b: &Deck) -> Self {
        Self {
            winner: None,
            points: [0, 0],
            turn_count: 0,
            current_player: 0,
            move_generation_stack: Arc::new(Vec::new()),
            current_energy: None,
            hands: [Arc::new(Vec::new()), Arc::new(Vec::new())],
            decks: [Arc::new(deck_a.clone()), Arc::new(deck_b.clone())],
            discard_piles: [Arc::new(Vec::new()), Arc::new(Vec::new())],
            in_play_pokemon: [[None, None, None, None], [None, None, None, None]],
            has_played_support: false,
            has_retreated: false,
            turn_effects: Arc::new(BTreeMap::new()),
        }
    }
    
    /// Get mutable access to a player's hand using copy-on-write
    pub fn hands_mut(&mut self, player: usize) -> &mut Vec<Card> {
        Arc::make_mut(&mut self.hands[player])
    }
    
    /// Get mutable access to a player's deck using copy-on-write
    pub fn decks_mut(&mut self, player: usize) -> &mut Deck {
        Arc::make_mut(&mut self.decks[player])
    }
    
    /// Get mutable access to a player's discard pile using copy-on-write
    pub fn discard_piles_mut(&mut self, player: usize) -> &mut Vec<Card> {
        Arc::make_mut(&mut self.discard_piles[player])
    }
    
    /// Get mutable access to move generation stack using copy-on-write
    pub fn move_generation_stack_mut(&mut self) -> &mut Vec<(usize, Vec<SimpleAction>)> {
        Arc::make_mut(&mut self.move_generation_stack)
    }
    
    /// Get mutable access to turn effects using copy-on-write
    pub fn turn_effects_mut(&mut self) -> &mut BTreeMap<u8, Vec<Card>> {
        Arc::make_mut(&mut self.turn_effects)
    }
    
    /// Remove a card from hand with copy-on-write
    pub(crate) fn remove_card_from_hand(&mut self, current_player: usize, card: &Card) {
        let hand = self.hands_mut(current_player);
        let index = hand
            .iter()
            .position(|x| x == card)
            .expect("Player hand should contain card to remove");
        hand.swap_remove(index);
    }
    
    /// Discard a card from hand with copy-on-write
    pub(crate) fn discard_card_from_hand(&mut self, current_player: usize, card: &Card) {
        self.remove_card_from_hand(current_player, card);
        self.discard_piles_mut(current_player).push(card.clone());
    }
    
    /// Draw a card with copy-on-write
    pub(crate) fn maybe_draw_card(&mut self, player: usize) {
        let deck = self.decks_mut(player);
        if let Some(card) = deck.draw() {
            let card_clone = card.clone();
            let card_name = canonical_name(&card);
            debug!(
                "Player {} drew: {:?}, deck has {} cards",
                player + 1,
                card_name,
                deck.cards.len()
            );
            
            self.hands_mut(player).push(card_clone);
            
            debug!(
                "Player {} hand is now: {:?}",
                player + 1,
                to_canonical_names(&self.hands[player])
            );
        } else {
            debug!("Player {} cannot draw a card, deck is empty", player + 1);
        }
    }
    
    /// Generate energy for current player
    pub(crate) fn generate_energy(&mut self) {
        let current_player = self.current_player;
        let deck_energies = &self.decks[current_player].energy_types;
        
        if deck_energies.len() == 1 {
            self.current_energy = Some(deck_energies[0]);
            return;
        }
        
        let mut rng = rand::thread_rng();
        let generated = deck_energies
            .choose(&mut rng)
            .expect("Decks should have at least 1 energy");
        self.current_energy = Some(*generated);
    }
    
    /// Reset turn states
    pub(crate) fn reset_turn_states(&mut self) {
        // Reset .played_this_turn and .ability_used for all in-play pokemon
        for i in 0..2 {
            self.in_play_pokemon[i].iter_mut().for_each(|x| {
                if let Some(pokemon) = x {
                    pokemon.played_this_turn = false;
                    pokemon.ability_used = false;
                }
            });
        }
        
        self.has_played_support = false;
        self.has_retreated = false;
    }
    
    /// Add a turn effect with copy-on-write
    pub(crate) fn add_turn_effect(&mut self, card: Card, duration: u8) {
        let current_turn = self.turn_count;
        let effects = self.turn_effects_mut();
        for turn_offset in 0..(duration + 1) {
            let target_turn = current_turn + turn_offset;
            effects
                .entry(target_turn)
                .or_default()
                .push(card.clone());
            trace!(
                "Adding effect {:?} for {} turns, current turn: {}, target turn: {}",
                canonical_name(&card),
                duration,
                current_turn,
                target_turn
            );
        }
    }
    
    /// Get current turn effects
    pub(crate) fn get_current_turn_effects(&self) -> Vec<Card> {
        self.turn_effects
            .get(&self.turn_count)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Queue draw action with copy-on-write
    pub(crate) fn queue_draw_action(&mut self, actor: usize) {
        self.move_generation_stack_mut()
            .push((actor, vec![SimpleAction::DrawCard]));
    }
    
    /// Advance turn
    pub(crate) fn advance_turn(&mut self) {
        debug!(
            "Ending turn moving from player {} to player {}",
            self.current_player,
            (self.current_player + 1) % 2
        );
        self.current_player = (self.current_player + 1) % 2;
        self.turn_count += 1;
        self.reset_turn_states();
        self.queue_draw_action(self.current_player);
        self.generate_energy();
    }
    
    // The rest of the methods can remain similar, just using the Arc fields appropriately
    
    pub fn get_remaining_hp(&self, player: usize, index: usize) -> u32 {
        self.in_play_pokemon[player][index]
            .as_ref()
            .unwrap()
            .remaining_hp
    }
    
    pub fn enumerate_in_play_pokemon(
        &self,
        player: usize,
    ) -> impl Iterator<Item = (usize, &PlayedCard)> {
        self.in_play_pokemon[player]
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .map(|(i, x)| (i, x.as_ref().unwrap()))
    }
    
    pub fn enumerate_bench_pokemon(
        &self,
        player: usize,
    ) -> impl Iterator<Item = (usize, &PlayedCard)> {
        self.enumerate_in_play_pokemon(player)
            .filter(|(i, _)| *i != 0)
    }
    
    pub(crate) fn get_active(&self, player: usize) -> &PlayedCard {
        self.in_play_pokemon[player][0]
            .as_ref()
            .expect("Active Pokemon should be there")
    }
    
    pub(crate) fn get_active_mut(&mut self, player: usize) -> &mut PlayedCard {
        self.in_play_pokemon[player][0]
            .as_mut()
            .expect("Active Pokemon should be there")
    }
    
    pub(crate) fn is_game_over(&self) -> bool {
        self.winner.is_some() || self.turn_count >= 100
    }
    
    pub(crate) fn num_in_play_of_type(&self, player: usize, energy: EnergyType) -> usize {
        self.enumerate_in_play_pokemon(player)
            .filter(|(_, x)| x.get_energy_type() == Some(energy))
            .count()
    }
    
    pub fn debug_string(&self) -> String {
        format!(
            "P1 Hand:\t{:?}\n\
            P1 InPlay:\t{:?}\n\
            P2 InPlay:\t{:?}\n\
            P2 Hand:\t{:?}",
            to_canonical_names(self.hands[0].as_slice()),
            format_cards(&self.in_play_pokemon[0]),
            format_cards(&self.in_play_pokemon[1]),
            to_canonical_names(self.hands[1].as_slice())
        )
    }
    
    pub fn initialize(deck_a: &Deck, deck_b: &Deck, rng: &mut impl Rng) -> Self {
        let mut state = Self::new(deck_a, deck_b);
        
        // Shuffle the decks before starting the game
        for i in 0..2 {
            let deck = state.decks_mut(i);
            deck.shuffle(true, rng);
        }
        
        // Draw 5 cards each
        for _ in 0..5 {
            state.maybe_draw_card(0);
            state.maybe_draw_card(1);
        }
        
        // Flip a coin to determine the starting player
        state.current_player = rng.gen_range(0..2);
        
        state
    }
}

// Helper functions
fn format_cards(played_cards: &[Option<PlayedCard>]) -> Vec<String> {
    played_cards.iter().map(format_card).collect()
}

fn format_card(x: &Option<PlayedCard>) -> String {
    match x {
        Some(played_card) => format!(
            "{}({}hp,{:?})",
            played_card.get_name(),
            played_card.remaining_hp,
            played_card.attached_energy.len(),
        ),
        None => "".to_string(),
    }
}

fn canonical_name(card: &Card) -> &String {
    match card {
        Card::Pokemon(pokemon_card) => &pokemon_card.name,
        Card::Trainer(trainer_card) => &trainer_card.name,
    }
}

fn to_canonical_names(cards: &[Card]) -> Vec<&String> {
    cards.iter().map(canonical_name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::load_test_decks;
    
    #[test]
    fn test_cheap_cloning() {
        let (deck_a, deck_b) = load_test_decks();
        let state1 = OptimizedState::new(&deck_a, &deck_b);
        
        // Clone should be cheap
        let state2 = state1.clone();
        
        // Arc pointers should be the same
        assert!(Arc::ptr_eq(&state1.hands[0], &state2.hands[0]));
        assert!(Arc::ptr_eq(&state1.decks[0], &state2.decks[0]));
        assert!(Arc::ptr_eq(&state1.discard_piles[0], &state2.discard_piles[0]));
    }
    
    #[test]
    fn test_copy_on_write() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state1 = OptimizedState::new(&deck_a, &deck_b);
        let state2 = state1.clone();
        
        // Modify state1's hand
        state1.maybe_draw_card(0);
        
        // Arc pointers should now be different for hands[0]
        assert!(!Arc::ptr_eq(&state1.hands[0], &state2.hands[0]));
        
        // But other fields should still share Arc
        assert!(Arc::ptr_eq(&state1.hands[1], &state2.hands[1]));
        assert!(Arc::ptr_eq(&state1.discard_piles[0], &state2.discard_piles[0]));
    }
    
    #[test]
    fn test_performance_comparison() {
        use std::time::Instant;
        
        let (deck_a, deck_b) = load_test_decks();
        
        // Test original State cloning
        let original_state = crate::state::State::new(&deck_a, &deck_b);
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = original_state.clone();
        }
        let original_duration = start.elapsed();
        
        // Test optimized State cloning
        let optimized_state = OptimizedState::new(&deck_a, &deck_b);
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = optimized_state.clone();
        }
        let optimized_duration = start.elapsed();
        
        println!("Original clone time: {:?}", original_duration);
        println!("Optimized clone time: {:?}", optimized_duration);
        
        // Optimized should be significantly faster
        assert!(optimized_duration < original_duration / 2);
    }
}