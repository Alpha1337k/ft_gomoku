use crate::{block_position::BlockPosition, constants::{B0_SCORES, B1_SCORES, B2_SCORES}, piece::Piece, position::Position};

#[derive(Clone, Debug)]
pub struct Line {
	pub id: usize,
	pub start: Position,
	pub end: Position,
	pub length: usize,
	pub direction: u8,
	pub score: f32,
	pub block_pos: BlockPosition,
	pub player: Piece,
}

impl Line {
	pub fn new(idx: usize, player: Piece, block_pos: BlockPosition, start: Position, end: Position, direction: u8, length: usize) -> Line {
		Line {
			id: idx,
			start,
			end,
			player,
			block_pos,
			direction,
			length,
			score: Self::calculate(block_pos, length, player),
		}
	}

	pub fn calculate(blocks: BlockPosition, length: usize, player: Piece) -> f32 {
		let val = match blocks {
			BlockPosition::NONE => B0_SCORES[length.min(5)],
			BlockPosition::START => B1_SCORES[length.min(5)],
			BlockPosition::END => B1_SCORES[length.min(5)],
			BlockPosition::BOTH => B2_SCORES[length.min(5)],
		};

		val * if player == Piece::Max {1.0} else {-1.0}
	}
}