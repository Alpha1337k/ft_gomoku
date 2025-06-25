



use crate::{constants::DIRECTIONS, heuristic::Heuristic, piece::{Piece, PieceWrap}, position::Position};

impl Heuristic<'_> {
	fn validate_free_two_line_requirements(&self, lines: &[usize; 4], direction_idx: usize, player: Piece) -> bool {
		let line = &self.lines.get_id(&lines[direction_idx]);
			
		if line.is_none() {
			return false;
		}

		line.unwrap().block_pos == 0 && line.unwrap().player == player && line.unwrap().length == 2
	}

	pub fn get_invalid_moves(&self, player: Piece) -> Vec<Position> {
		let mut positions = Vec::new();

		for pos in self.board {
			if self.board[&pos].is_empty() && !self.validate_move(pos, player) {
				positions.push(pos);
			}
		}

		positions
	}

	fn is_free_two(&self, mut pos: Position, direction: [i32; 2], direction_idx: usize, player: Piece) -> i32 {
		if pos.relocate(direction[0], direction[1]).is_err() {
			return -1
		}
		if self.lines.get_pos(&pos)
			.is_some_and(|lines| self.validate_free_two_line_requirements(lines, direction_idx, player)) {
			return 1;
		}

		if self.board[&pos].is_piece() {
			return -1;
		}

		if pos.relocate(direction[0], direction[1]).is_err() {
			return 0
		}

		if self.lines.get_pos(&pos)
			.is_some_and(|lines| self.validate_free_two_line_requirements(lines, direction_idx, player)) {
			return 1;
		}

		0
	}

	pub fn validate_move(&self, pos: Position, player: Piece) -> bool {
		// We needs to check if the construction of a free two is possible, per requirements in subject.

		let mut free_three_count = 0;
		for (i, direction) in DIRECTIONS.iter().enumerate() {
			let results = (
				self.is_free_two(pos, direction[0], i, player),
				self.is_free_two(pos, direction[1], i, player)
			);

			if results.0 == -1 || results.1 == -1 || results.0 + results.1 != 1 {
				continue;
			}

			free_three_count += 1;
		}

		free_three_count != 2
	}
}