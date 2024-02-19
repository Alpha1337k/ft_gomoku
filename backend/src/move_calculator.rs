use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{board::Board, piece::{Piece, PieceWrap}, position::Position};

const DIRECTIONS: [[[i32; 2]; 2]; 4] = [
	[[-1, 0], [1, 0]], //x
	[[0, -1], [0, 1]], //y
	[[-1, -1], [1, 1]], //tlbr
	[[-1, 1], [1, -1]], //trbl
];

#[derive(Serialize, Deserialize)]
pub struct Move {
	position: Position,
	piece: Piece
}

#[derive(Serialize, Deserialize)]
pub struct MoveCalculator {
	pub positions_checked: usize,
	pub moves: [HashSet<Position>; 2]
}

impl MoveCalculator {
	pub fn new(board: &Board) -> MoveCalculator {
		let mut new_calc = MoveCalculator{
			moves: [HashSet::new(), HashSet::new()],
			positions_checked: 0,
		};

		for pos in board {
			if board[&pos].is_piece() {
				continue;
			}

			if new_calc.validate_virtual_move(pos, Piece::Max, board) {
				new_calc.moves[Piece::Max as usize].insert(pos);
			}
			if new_calc.validate_virtual_move(pos, Piece::Min, board) {
				new_calc.moves[Piece::Min as usize].insert(pos);
			}
		}

		return new_calc
	}

	pub fn from_calculator(old: &MoveCalculator, board: &Board, delta: Vec<Move>) -> MoveCalculator {
		let mut new_calc = MoveCalculator{
			moves: old.moves.clone(),
			positions_checked: 0,
		};

		let mut checkable_positions = HashSet::<Position>::with_capacity(delta.len() * 25);

		for d in delta {
			for x in -1..1 {
				for y in -1..1 {
					if x == 0 && y == 0 && d.piece != Piece::Empty {
						continue;
					}

					let mut new_pos = d.position.clone();

					if new_pos.relocate(x, y).is_err() {
						continue;
					}

					checkable_positions.insert(new_pos);
				}	
			}
		}

		for pos in checkable_positions {
			if new_calc.validate_virtual_move(pos, Piece::Max, board) {
				new_calc.moves[Piece::Max as usize].insert(pos);
			} else {
				new_calc.moves[Piece::Max as usize].remove(&pos);
			}
			if new_calc.validate_virtual_move(pos, Piece::Min, board) {
				new_calc.moves[Piece::Min as usize].insert(pos);
			} else {
				new_calc.moves[Piece::Min as usize].remove(&pos);
			}
		}

		
		return new_calc;
	}

	fn match_pattern(&self, start_pos: Position, direction: [i32;2], pattern: &Vec<Piece>, board: &Board) -> bool {
		let mut pos = start_pos.clone();

		for i in 0..pattern.len() {
			if board[&pos].is_equal(&pattern[i]) == false {
				return false;
			}
			if pos.relocate(direction[0], direction[1]).is_err() {
				return false;
			}
		}

		return true;
	}

	fn validate_virtual_move(&mut self, pos: Position, player: Piece, board: &Board) -> bool
	{
		self.positions_checked += 1;

		let capture_pattern = vec![
			player, player.get_opposite(), player.get_opposite(), player
		];

		let patterns = vec![
			vec![Piece::Empty, player, player, player, Piece::Empty],
			vec![Piece::Empty, player, player, Piece::Empty, player, Piece::Empty],
			vec![Piece::Empty, player, Piece::Empty, player, player, Piece::Empty],
		];

		for direction in DIRECTIONS {
			if	self.match_pattern(pos, direction[0], &capture_pattern, board) ||
				self.match_pattern(pos, direction[1], &capture_pattern, board)
			{
				return true;
			}

			for offset in -5i32..4 {
				let mut cur_pos = pos.clone();

				if offset < 0 {
					if cur_pos.relocate_n(direction[0][0], direction[0][1], offset.abs() as usize).is_err() {
						continue;
					}
				} else if offset > 0 {
					if cur_pos.relocate_n(direction[1][0], direction[1][1], offset as usize).is_err() {
						continue;
					}		
				}

				if	self.match_pattern(cur_pos, direction[1], &patterns[0], board) ||
					self.match_pattern(cur_pos, direction[1], &patterns[1], board) ||
					self.match_pattern(cur_pos, direction[1], &patterns[2], board) {
						return false;
					}
			}
		}
		return true;
	}
}