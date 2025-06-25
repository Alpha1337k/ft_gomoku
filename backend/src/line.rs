use crate::{constants::{B0_SCORES, B1_SCORES, B2_SCORES}, piece::Piece, position::Position};

#[derive(Clone, Debug)]
pub struct Line {
	pub id: usize,
	pub start: Position,
	pub end: Position,
	pub length: usize,
	pub direction: u8,
	pub score: f32,
	pub block_pos: u8,
	pub player: Piece,
}

impl Line {
	pub fn new(idx: usize, player: Piece, blocks: u8, start: Position, end: Position, direction: u8, length: usize) -> Line {
		Line {
			id: idx,
			start,
			end,
			player,
			block_pos: blocks,
			direction,
			length,
			score: Self::calculate(blocks, length, player),
		}
	}

	pub fn calculate(blocks: u8, length: usize, player: Piece) -> f32 {
		let val = match blocks {
			0 => B0_SCORES[length.min(5)],
			1 => B1_SCORES[length.min(5)],
			2 => B1_SCORES[length.min(5)],
			3 => B2_SCORES[length.min(5)],
			_ => f32::INFINITY
		};

		val * if player == Piece::Max {1.0} else {-1.0}
	}
}