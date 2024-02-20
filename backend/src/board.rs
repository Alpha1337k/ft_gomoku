


use serde::Serialize;
use serde_json::Value;
use crate::{move_calculator::Move, piece::{Piece, PieceWrap}, position::Position};

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

	pub fn get_delta(old: &Board, n: &Board) -> Vec<Move> {
		let mut rv = Vec::new();
		
		for pos in old {
			if old[&pos] != n[&pos] {
				rv.push(Move{
					position: pos,
					piece: n[&pos]
				})
			}
		}

		return rv;
	}

	pub fn get_captures(board: &Board, pos: Position, player: Piece) -> u8 {
		let directions = [
			[[-1, 0], [-2, 0], [-3, 0]],
			[[1, 0], [2, 0], [3, 0]],
			[[0, 1], [0, 2], [0, 3]],
			[[0, -1], [0, -2], [0, -3]],
			[[-1, -1], [-2, -2], [-3, -3]],
			[[1, 1], [2, 2], [3, 3]],
			[[-1, 1], [-2, 2], [-3, 3]],
			[[1, 1], [2, -2], [3, -3]],
		];

		let mut rv = 0;

		for (i, direction) in directions.iter().enumerate() {
			if pos.clone().relocate(direction[0][0], direction[0][1]).is_ok_and(|f| board[&f].is_opposite(&player)) &&
				pos.clone().relocate(direction[1][0], direction[1][1]).is_ok_and(|f| board[&f].is_opposite(&player)) &&
				pos.clone().relocate(direction[2][0], direction[2][1]).is_ok_and(|f| board[&f].is_equal(&player)) {
				rv |= 1u8 << i;
			}
		}

		return rv;
	}

	pub fn set_move(&mut self, pos: Position, player: Piece, capture_map: Option<u8>) -> &Board {
		self[&pos] = player;

		if capture_map.is_some_and(|x| x == 0) {
			return self;
		}

		let mut captures = capture_map.unwrap_or_else(|| Self::get_captures(&self, pos, player));

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

				println!("TAKING FOR IDX {} MOVE {}", map_idx, pos);

				if (
					pos.clone().relocate(map[0][0], map[0][1]).is_ok_and(|x| self.data[x.x + x.y * 19].is_opposite(&player)) && 
					pos.clone().relocate(map[1][0], map[1][1]).is_ok_and(|x| self.data[x.x + x.y * 19].is_opposite(&player))
				) {
					self[&pos.clone().relocate(map[0][0], map[0][1]).unwrap()] = Piece::Empty;
					self[&pos.clone().relocate(map[1][0], map[1][1]).unwrap()] = Piece::Empty;
				} else {
					panic!();
				}

			}
			captures >>= 1;
			map_idx += 1;
		}

		self
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

		if self.index >= 361 {
			return None;			
		}

		return Some(Position::from_u64(self.index));
	}
}