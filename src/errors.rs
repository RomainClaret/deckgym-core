use std::fmt;

/// Comprehensive error type for all game-related errors.
/// This replaces panic-prone unwrap/expect calls throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    // State-related errors
    InvalidCardPosition { position: usize, max: usize },
    CardNotInHand { card_name: String, player: usize },
    NoPokemonAtPosition { player: usize, position: usize },
    NoActivePokemon { player: usize },
    EmptyDeck { player: usize },
    InvalidPlayer { player: usize },
    
    // Action-related errors
    InvalidAction { action: String, reason: String },
    IllegalMove { description: String },
    InvalidEvolution { reason: String },
    InvalidAttachment { reason: String },
    
    // Card-related errors
    CardNotFound { card_id: u16 },
    InvalidCardType { expected: String, found: String },
    MissingEnergy { required: Vec<String>, available: Vec<String> },
    
    // Game state errors
    GameAlreadyOver,
    InvalidGameState { description: String },
    NoLegalMoves { player: usize },
    
    // Deck errors
    InvalidDeckFormat { reason: String },
    DeckValidationFailed { errors: Vec<String> },
    InsufficientCards { required: usize, found: usize },
    
    // Configuration errors
    InvalidConfiguration { setting: String, value: String },
    MissingRequiredField { field: String },
    
    // AI/Player errors
    PlayerError { player_type: String, error: String },
    AICalculationError { description: String },
    
    // Hook errors
    HookExecutionFailed { hook_name: String, error: String },
    
    // Generic errors for unexpected situations
    InternalError { context: String, details: String },
    NotImplemented { feature: String },
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::InvalidCardPosition { position, max } => {
                write!(f, "Invalid card position {}, max allowed is {}", position, max)
            }
            GameError::CardNotInHand { card_name, player } => {
                write!(f, "Card '{}' not found in player {}'s hand", card_name, player + 1)
            }
            GameError::NoPokemonAtPosition { player, position } => {
                write!(f, "No Pokemon at position {} for player {}", position, player + 1)
            }
            GameError::NoActivePokemon { player } => {
                write!(f, "Player {} has no active Pokemon", player + 1)
            }
            GameError::EmptyDeck { player } => {
                write!(f, "Player {}'s deck is empty", player + 1)
            }
            GameError::InvalidPlayer { player } => {
                write!(f, "Invalid player index: {}", player)
            }
            GameError::InvalidAction { action, reason } => {
                write!(f, "Invalid action '{}': {}", action, reason)
            }
            GameError::IllegalMove { description } => {
                write!(f, "Illegal move: {}", description)
            }
            GameError::InvalidEvolution { reason } => {
                write!(f, "Invalid evolution: {}", reason)
            }
            GameError::InvalidAttachment { reason } => {
                write!(f, "Invalid attachment: {}", reason)
            }
            GameError::CardNotFound { card_id } => {
                write!(f, "Card with ID {} not found", card_id)
            }
            GameError::InvalidCardType { expected, found } => {
                write!(f, "Invalid card type: expected {}, found {}", expected, found)
            }
            GameError::MissingEnergy { required, available } => {
                write!(f, "Missing energy: required {:?}, available {:?}", required, available)
            }
            GameError::GameAlreadyOver => {
                write!(f, "Game is already over")
            }
            GameError::InvalidGameState { description } => {
                write!(f, "Invalid game state: {}", description)
            }
            GameError::NoLegalMoves { player } => {
                write!(f, "Player {} has no legal moves", player + 1)
            }
            GameError::InvalidDeckFormat { reason } => {
                write!(f, "Invalid deck format: {}", reason)
            }
            GameError::DeckValidationFailed { errors } => {
                write!(f, "Deck validation failed: {}", errors.join(", "))
            }
            GameError::InsufficientCards { required, found } => {
                write!(f, "Insufficient cards: required {}, found {}", required, found)
            }
            GameError::InvalidConfiguration { setting, value } => {
                write!(f, "Invalid configuration: {} = '{}'", setting, value)
            }
            GameError::MissingRequiredField { field } => {
                write!(f, "Missing required field: {}", field)
            }
            GameError::PlayerError { player_type, error } => {
                write!(f, "Player error ({}): {}", player_type, error)
            }
            GameError::AICalculationError { description } => {
                write!(f, "AI calculation error: {}", description)
            }
            GameError::HookExecutionFailed { hook_name, error } => {
                write!(f, "Hook '{}' execution failed: {}", hook_name, error)
            }
            GameError::InternalError { context, details } => {
                write!(f, "Internal error in {}: {}", context, details)
            }
            GameError::NotImplemented { feature } => {
                write!(f, "Feature not implemented: {}", feature)
            }
        }
    }
}

impl std::error::Error for GameError {}

/// Type alias for Results in the game engine
pub type GameResult<T> = Result<T, GameError>;

/// Helper trait to convert Option to Result with context
pub trait OptionExt<T> {
    fn ok_or_game_error<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> GameError;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_game_error<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> GameError,
    {
        self.ok_or_else(f)
    }
}

/// Helper functions for common error patterns
impl GameError {
    pub fn invalid_position(position: usize, max: usize) -> Self {
        GameError::InvalidCardPosition { position, max }
    }
    
    pub fn card_not_in_hand(card_name: impl Into<String>, player: usize) -> Self {
        GameError::CardNotInHand {
            card_name: card_name.into(),
            player,
        }
    }
    
    pub fn no_pokemon(player: usize, position: usize) -> Self {
        GameError::NoPokemonAtPosition { player, position }
    }
    
    pub fn no_active(player: usize) -> Self {
        GameError::NoActivePokemon { player }
    }
    
    pub fn invalid_action(action: impl Into<String>, reason: impl Into<String>) -> Self {
        GameError::InvalidAction {
            action: action.into(),
            reason: reason.into(),
        }
    }
    
    pub fn internal(context: impl Into<String>, details: impl Into<String>) -> Self {
        GameError::InternalError {
            context: context.into(),
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = GameError::InvalidCardPosition { position: 5, max: 3 };
        assert_eq!(err.to_string(), "Invalid card position 5, max allowed is 3");
        
        let err = GameError::CardNotInHand {
            card_name: "Pikachu".to_string(),
            player: 0,
        };
        assert_eq!(err.to_string(), "Card 'Pikachu' not found in player 1's hand");
    }
    
    #[test]
    fn test_option_ext() {
        let result: GameResult<i32> = None.ok_or_game_error(|| GameError::EmptyDeck { player: 0 });
        assert!(result.is_err());
        
        let result: GameResult<i32> = Some(42).ok_or_game_error(|| GameError::EmptyDeck { player: 0 });
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_helper_functions() {
        let err = GameError::invalid_position(5, 3);
        match err {
            GameError::InvalidCardPosition { position, max } => {
                assert_eq!(position, 5);
                assert_eq!(max, 3);
            }
            _ => panic!("Wrong error type"),
        }
    }
}