mod ability_ids;
pub mod actions;
mod attack_ids;
pub mod card_ids;
pub mod database;
pub mod deck;
mod errors;
pub mod game;
mod hidden_information;
mod hooks;
pub mod move_generation;
mod optimize;
mod optimized_state;
pub mod players;
mod safe_state;
pub mod simulate;
pub mod state;
pub mod test_helpers; // TODO: Compile/Expose only in test mode?
pub mod tool_ids;
pub mod types;

pub use ability_ids::AbilityId;
pub use attack_ids::AttackId;
pub use deck::Deck;
pub use game::Game;
pub use move_generation::generate_possible_actions;
pub use move_generation::generate_possible_trainer_actions;
pub use optimize::optimize;
pub use simulate::simulate;
pub use state::State;

// Error handling
pub use errors::{GameError, GameResult};
