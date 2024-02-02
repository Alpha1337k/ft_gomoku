use crate::board::{Board, Position};


pub struct Heuristic<'a> {
	board: &'a Board,
}

impl Heuristic<'_> {
	pub fn from_board(board: &Board) -> Heuristic {
		Heuristic {
			board: board
		}
	}

	fn get_position_score(pos: Position) -> f32 {
		let y = 1f32 - ((9.5 - (pos.y as f32)).abs() / 9.5f32);
		let x = 1f32 - ((9.5 - (pos.x % 19) as f32).abs() / 9.5f32);

		return (y + x) / 2f32;
	}

	fn get_position_scores(board: &Board) -> f32 {
		let mut score = 0.0;
		
		for y in 0..19 {
			for x in 0..19 {
				let pos = Position::new(x, y);
				if board[&pos].is_max() {
					score += Self::get_position_score(pos);
				} else if board[&pos].is_min() {
					score -= Self::get_position_score(pos);
				}
			}
		}

		return score;
	}

	pub fn get_move_order(&self, is_maximizing: bool) -> Vec<Position> {

	}
}