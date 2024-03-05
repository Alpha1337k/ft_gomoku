


use std::{fmt};

use serde::Serialize;
use serde_json::Value;
use crate::{piece::{Piece, PieceWrap}, position::Position};

#[derive(Clone, Serialize)]
pub struct Board {
	data: Vec<Piece>,
}

impl Board {
	pub fn new() -> Board {
		Board { 
			data: vec![Piece::Empty; 19 * 19],
		}
	}

	pub fn from_map(board_map: &serde_json::Map<String, Value>) -> Board {
		let mut board =  Board { 
			data: vec![Piece::Empty; 19 * 19],
		 };

		 for (key, value) in board_map {
			let key_int = key.parse::<usize>().unwrap();
			board.data[key_int] = value.as_u64().unwrap().try_into().unwrap_or_default();
		}

		return board;
	}

	pub fn get_diff(b1: &Board, b2: &Board) -> Vec<Position> {
		let mut diffs = Vec::with_capacity(4);
		
		for pos in b1 {
			if b1[&pos] != b2[&pos] {
				diffs.push(pos);
			}
		}

		return diffs;
	}

	pub fn get_captures(board: &Board, pos: Position, player: Piece) -> u8 {
		let directions = [
			[[-1, 0], [-2, 0], [-3, 0]],
			[[1, 0], [2, 0], [3, 0]],
			[[0, -1], [0, -2], [0, -3]],
			[[0, 1], [0, 2], [0, 3]],
			[[-1, -1], [-2, -2], [-3, -3]],
			[[1, 1], [2, 2], [3, 3]],
			[[-1, 1], [-2, 2], [-3, 3]],
			[[1, -1], [2, -2], [3, -3]],
		];

		let mut rv = 0;

		for (i, direction) in directions.iter().enumerate() {
			if pos.clone().relocate(direction[0][0], direction[0][1]).is_ok_and(|f| board[&f] == player.get_opposite()) &&
				pos.clone().relocate(direction[1][0], direction[1][1]).is_ok_and(|f| board[&f] == player.get_opposite()) &&
				pos.clone().relocate(direction[2][0], direction[2][1]).is_ok_and(|f| board[&f].is_equal(&player)) {
				rv |= 1u8 << i;
			}
		}

		println!("GET CAPTURES RES: {}, P: {}", rv, pos);

		return rv;
	}

	pub fn set_move(&mut self, pos: Position, player: Piece, capture_map: Option<u8>) -> usize {
		if self[&pos].is_piece() {
			panic!();
		}

		self[&pos] = player;

		if capture_map.is_some_and(|x| x == 0) {
			return 0;
		}

		let mut captures = capture_map.unwrap_or_else(|| Self::get_captures(&self, pos, player));
		let captures_store = captures;

		let mut capture_count = 0;

		let maps = [
			[[-1, 0], [-2, 0]],
			[[1, 0], [2, 0]],
			[[0, -1], [0, -2]],
			[[0, 1], [0, 2]],
			[[-1, -1], [-2, -2]],
			[[1, 1], [2, 2]],
			[[-1, 1], [-2, 2]],
			[[1, -1], [2, -2]],
		];

		let mut map_idx = 0;
		while captures != 0 {
			let needs_capture = captures & 0x1;
			if needs_capture == 1 {
				let map = maps[map_idx];
				// println!("TAKING FOR IDX {} MOVE {}", map_idx, pos);

				if pos.clone().relocate(map[0][0], map[0][1]).is_ok_and(|x| self[x] == player.get_opposite()) && 
					pos.clone().relocate(map[1][0], map[1][1]).is_ok_and(|x| self[x] == player.get_opposite()) {
					self[&pos.clone().relocate(map[0][0], map[0][1]).unwrap()] = Piece::Empty;
					self[&pos.clone().relocate(map[1][0], map[1][1]).unwrap()] = Piece::Empty;
				} else {
					println!("\nFAILED CAPTURE AT POS: {} ({})", pos, captures_store);
					dbg!(player, map[0], map[1]);
					println!("{}", self);
					panic!();
				}

				capture_count += 1;

			}
			captures >>= 1;
			map_idx += 1;
		}

		capture_count
	}

	pub fn get(&self, x: usize, y: usize) -> &Piece {
		return &self.data[y * 19 + x];
	}

	pub fn len(&self) -> usize {
		return self.data.len();
	}
}

impl std::ops::Index<&Position> for Board {
    type Output = Piece;

    fn index(&self, idx: &Position) -> &Piece {
		return &self.data[idx.x + idx.y * 19];
    }
}

impl std::ops::IndexMut<&Position> for Board {
    fn index_mut(&mut self, idx: &Position) -> &mut Piece {
		return &mut self.data[idx.x + idx.y * 19];
    }
}

pub struct BoardIterator {
	index: usize
}

impl<'a> IntoIterator for &'a Board {
    type Item = Position;
    type IntoIter = BoardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BoardIterator {
            index: 0,
        }
    }
}


impl<'a> Iterator for BoardIterator {
	type Item = Position;

	fn next(&mut self) -> Option<Self::Item> {
		self.index += 1;

		if self.index >= 362 {
			return None;			
		}

		return Some(Position::from_u64(self.index - 1));
	}
}

impl fmt::Display for Board {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "  ")?;
		for i in 0..19 {
			write!(f, "{:3}", i)?;
		}
		for i in 0..19*19 {
			if i % 19 == 0 {
				write!(f, "\n{}:", ((i as f64 / 19.0).floor() as u8 + 65) as char )?;
			}
			write!(f, " {} ", self.data[i])?;
		}
		Ok(())
	}
}