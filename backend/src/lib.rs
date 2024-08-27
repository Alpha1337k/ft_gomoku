use  serde::Deserialize;
use  serde_json::Value;

pub mod minimax;
pub mod board;
pub mod position;
pub mod piece;
pub mod heuristic;
pub mod heuristic_v2;
pub mod move_calculator;

#[derive(Deserialize)]
pub struct CalculateRequest {
	pub board: serde_json::Map<String, Value>,
	pub depth: usize,
	pub in_move: Option<position::Position>,
	pub player: piece::Piece,
	pub captures: [usize; 2],
	pub is_hint: Option<bool>
}