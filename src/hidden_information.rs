use std::collections::HashMap;
use crate::{
    types::{Card, EnergyType},
    State,
};

/// Represents the knowledge a player has about hidden game information
#[derive(Debug, Clone, PartialEq)]
pub struct HiddenKnowledge {
    /// What each player knows about deck contents (card -> count)
    pub known_deck_contents: [HashMap<Card, usize>; 2],
    
    /// What each player knows about opponent's hand
    pub known_hand_sizes: [usize; 2],
    
    /// Track if a player has perfect information (for human players)
    pub has_perfect_info: [bool; 2],
}

impl Default for HiddenKnowledge {
    fn default() -> Self {
        Self {
            known_deck_contents: [HashMap::new(), HashMap::new()],
            known_hand_sizes: [0, 0],
            has_perfect_info: [false, false],
        }
    }
}

impl HiddenKnowledge {
    /// Create initial knowledge from deck lists
    pub fn from_decklists(deck1_cards: &[Card], deck2_cards: &[Card]) -> Self {
        let mut knowledge = Self::default();
        
        // Each player knows the opponent's decklist
        for card in deck1_cards {
            *knowledge.known_deck_contents[1].entry(card.clone()).or_insert(0) += 1;
        }
        
        for card in deck2_cards {
            *knowledge.known_deck_contents[0].entry(card.clone()).or_insert(0) += 1;
        }
        
        // Initial hand sizes after drawing 5
        knowledge.known_hand_sizes = [5, 5];
        
        knowledge
    }
    
    /// Update knowledge when a card is played from hand
    pub fn card_played_from_hand(&mut self, player: usize, card: &Card) {
        let opponent = (player + 1) % 2;
        
        // Opponent knows this card was in hand, remove from possible deck cards
        if let Some(count) = self.known_deck_contents[opponent].get_mut(card) {
            *count = count.saturating_sub(1);
        }
        
        // Update hand size
        self.known_hand_sizes[player] = self.known_hand_sizes[player].saturating_sub(1);
    }
    
    /// Update knowledge when a card is drawn
    pub fn card_drawn(&mut self, player: usize) {
        self.known_hand_sizes[player] += 1;
    }
    
    /// Get probability distribution for what cards could be in deck
    pub fn get_deck_probabilities(&self, player: usize, observer: usize) -> HashMap<Card, f64> {
        if self.has_perfect_info[observer] || player == observer {
            // Perfect information or looking at own deck
            return HashMap::new();
        }
        
        let known_cards = &self.known_deck_contents[observer];
        let total_unknown: usize = known_cards.values().sum();
        
        if total_unknown == 0 {
            return HashMap::new();
        }
        
        known_cards
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|(card, &count)| {
                (card.clone(), count as f64 / total_unknown as f64)
            })
            .collect()
    }
}

/// Represents an action outcome without revealing hidden information
#[derive(Debug, Clone)]
pub enum HiddenOutcome {
    /// Draw cards without revealing what they are
    DrawCards { player: usize, count: usize },
    
    /// Search deck for a card matching criteria
    SearchDeck {
        player: usize,
        filter: DeckSearchFilter,
        max_cards: usize,
    },
    
    /// Shuffle hand into deck and draw
    ShuffleHandAndDraw { player: usize, draw_count: usize },
    
    /// Look at top card and make decision
    LookAtTopCard { player: usize, decision: TopCardDecision },
}

#[derive(Debug, Clone)]
pub enum DeckSearchFilter {
    Basic,
    Type(EnergyType),
    Specific(Card),
    Any,
}

#[derive(Debug, Clone)]
pub enum TopCardDecision {
    PutInHand,
    PutOnBottom,
    Discard,
}

/// Safe trainer effects that don't leak information
pub mod safe_trainer_effects {
    use super::*;
    use crate::{
        actions::{SimpleAction, Action},
        types::TrainerCard,
    };
    
    /// Safe Professor's Research effect - draws 2 without revealing cards
    pub fn safe_professor_research(state: &State, action: &Action) -> Vec<HiddenOutcome> {
        vec![HiddenOutcome::DrawCards {
            player: action.actor,
            count: 2,
        }]
    }
    
    /// Safe Pokeball effect - searches for basic without revealing options
    pub fn safe_pokeball(state: &State, action: &Action) -> Vec<HiddenOutcome> {
        vec![HiddenOutcome::SearchDeck {
            player: action.actor,
            filter: DeckSearchFilter::Basic,
            max_cards: 1,
        }]
    }
    
    /// Safe Red Card effect - opponent shuffles and draws without revealing
    pub fn safe_red_card(state: &State, action: &Action) -> Vec<HiddenOutcome> {
        let opponent = (action.actor + 1) % 2;
        vec![HiddenOutcome::ShuffleHandAndDraw {
            player: opponent,
            draw_count: 3,
        }]
    }
    
    /// Safe Mythical Slab effect - looks at top card without revealing
    pub fn safe_mythical_slab(state: &State, action: &Action) -> Vec<HiddenOutcome> {
        vec![HiddenOutcome::LookAtTopCard {
            player: action.actor,
            decision: TopCardDecision::PutInHand, // Simplified for now
        }]
    }
}

/// Converts hidden outcomes to concrete actions when needed
pub fn resolve_hidden_outcome(
    outcome: &HiddenOutcome,
    state: &State,
    rng: &mut impl rand::Rng,
) -> Vec<SimpleAction> {
    match outcome {
        HiddenOutcome::DrawCards { player, count } => {
            vec![SimpleAction::DrawCard; *count]
        }
        
        HiddenOutcome::SearchDeck { player, filter, max_cards } => {
            // When actually resolving, we can look at the deck
            let matching_cards: Vec<_> = state.decks[*player]
                .cards
                .iter()
                .filter(|card| match filter {
                    DeckSearchFilter::Basic => card.is_basic(),
                    DeckSearchFilter::Type(energy_type) => {
                        matches!(card, Card::Pokemon(p) if p.energy_type == *energy_type)
                    }
                    DeckSearchFilter::Specific(target) => *card == target,
                    DeckSearchFilter::Any => true,
                })
                .take(*max_cards)
                .cloned()
                .collect();
            
            // Convert to actions
            matching_cards
                .into_iter()
                .map(|_| SimpleAction::DrawCard) // Simplified
                .collect()
        }
        
        HiddenOutcome::ShuffleHandAndDraw { player, draw_count } => {
            // Actual implementation would shuffle first
            vec![SimpleAction::DrawCard; *draw_count]
        }
        
        HiddenOutcome::LookAtTopCard { player, decision } => {
            // Actual implementation would check top card
            vec![] // Simplified
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Pokemon, TrainerCard, TrainerType};
    
    #[test]
    fn test_hidden_knowledge_tracking() {
        let card1 = Card::Pokemon(Pokemon {
            id: 1,
            name: "Pikachu".to_string(),
            hp: 60,
            energy_type: EnergyType::Electric,
            stage: 0,
            evolves_from: None,
            weakness: None,
        });
        
        let card2 = Card::Trainer(TrainerCard {
            id: 2,
            name: "Potion".to_string(),
            trainer_type: TrainerType::Item,
        });
        
        let deck1 = vec![card1.clone(); 10];
        let deck2 = vec![card2.clone(); 10];
        
        let mut knowledge = HiddenKnowledge::from_decklists(&deck1, &deck2);
        
        // Player 0 knows player 1 has 10 Potions
        assert_eq!(knowledge.known_deck_contents[0][&card2], 10);
        
        // When player 1 plays a Potion
        knowledge.card_played_from_hand(1, &card2);
        
        // Player 0 knows there are now 9 Potions max in deck
        assert_eq!(knowledge.known_deck_contents[0][&card2], 9);
        assert_eq!(knowledge.known_hand_sizes[1], 4);
    }
    
    #[test]
    fn test_deck_probabilities() {
        let card1 = Card::Pokemon(Pokemon {
            id: 1,
            name: "Pikachu".to_string(),
            hp: 60,
            energy_type: EnergyType::Electric,
            stage: 0,
            evolves_from: None,
            weakness: None,
        });
        
        let card2 = Card::Pokemon(Pokemon {
            id: 2,
            name: "Raichu".to_string(),
            hp: 90,
            energy_type: EnergyType::Electric,
            stage: 1,
            evolves_from: Some("Pikachu".to_string()),
            weakness: None,
        });
        
        let mut knowledge = HiddenKnowledge::default();
        knowledge.known_deck_contents[0].insert(card1.clone(), 3);
        knowledge.known_deck_contents[0].insert(card2.clone(), 1);
        
        let probs = knowledge.get_deck_probabilities(1, 0);
        
        assert_eq!(probs[&card1], 0.75); // 3/4
        assert_eq!(probs[&card2], 0.25); // 1/4
    }
}