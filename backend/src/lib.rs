use  serde::Deserialize;
use  serde_json::Value;

pub mod minimax;
pub mod board;
pub mod position;
pub mod piece;
pub mod heuristic;
pub mod validate_move;
pub mod constants;
pub mod line;
pub mod line_store;

#[derive(Deserialize)]
pub struct CalculateRequest {
	pub board: serde_json::Map<String, Value>,
	pub depth: usize,
	pub in_move: Option<position::Position>,
	pub player: piece::Piece,
	pub captures: [usize; 2],
	pub is_hint: Option<bool>
}