use std::{fmt::{self}};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
pub enum Piece {
	Empty = -1,
	Max = 0,
	Min = 1
}

impl TryFrom<u64> for Piece {
	type Error = ();

	fn try_from(value: u64) -> Result<Self, Self::Error> {
		match value {
			x if x == Piece::Max as u64 => Ok(Piece::Max),
			x if x == Piece::Min as u64 => Ok(Piece::Min),
			_ => Err(())
		}
	}
}

impl fmt::Display for Piece {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let to_write = match self {
			Piece::Empty => 'E',
			Piece::Max => 'X',
			Piece::Min => 'I',
		};
		write!(f, "{}", to_write)
	}
}

impl Default for Piece {
    fn default() -> Self { Piece::Empty }
}

#[derive(Clone)]
pub struct Board {
	data: Vec<Piece>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
	pub x: usize,
	pub y: usize,
}

impl Position {
	pub fn new(x: usize, y: usize) -> Position {
		return *Position {
			x: x,
			y: y,
		}.check_pos().unwrap();
	}

	fn check_pos(&self) -> Result<&Position, &str> {
		if self.x >= 19 || self.y >= 19 {
			return Err("Invalid position");
		}
		return Ok(self);
	}

	pub fn relocate(&mut self, d_x: i32, d_y: i32) -> Result<&Self, &str> {
		self.x = self.x.wrapping_add(d_x as usize);
		self.y = self.y.wrapping_add(d_y as usize);

		return self.check_pos();
	}

	pub fn relocate_n(&mut self, d_x: i32, d_y: i32, n: usize) -> Result<&Self, &str> {
		for _i in 0..n {
			if self.relocate(d_x, d_y).is_err() {
				return Err("Invalid position");
			}
		}

		return Ok(self);
	}

	pub fn to_u64(&self) -> u64 {
		return (self.y * 19 + self.x) as u64;
	}

	pub fn from_u64(pos: usize) -> Position {
		return *Position {
			x: pos % 19,
			y: pos.div_euclid(19)
		}.check_pos().unwrap();
	}
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let y_char = (self.y as u8) + 65;
        write!(f, "{}{}", y_char as char, self.x)
    }
}

pub trait PieceWrap {
	fn is_max(&self) -> bool;
	fn is_min(&self) -> bool;
	fn is_piece(&self) -> bool;
	fn is_empty(&self) -> bool;
	fn is_opposite(&self, p: &Piece) -> bool;
	fn is_equal(&self, p: &Piece) -> bool;
}

impl PieceWrap for Piece {
	fn is_max(&self) -> bool {
		match self {
			Piece::Max => true,
			_ => false
		}
	}
	fn is_min(&self) -> bool {
		match self {
			Piece::Min => true,
			_ => false
		}
	}
	fn is_piece(&self) -> bool {
		match self {
			Piece::Empty => false,
			_ => true
		}
	}
	fn is_empty(&self) -> bool {
		match self {
			Piece::Empty => true,
			_ => false
		}
	}
	// check if is opposite player or empty
	fn is_opposite(&self, p: &Piece) -> bool {
		match self {
			Piece::Empty => true,
			Piece::Max => p != &Piece::Max,
			Piece::Min => p != &Piece::Min,
		}
	}
	fn is_equal(&self, p: &Piece) -> bool {
		match self {
			Piece::Empty => false,
			Piece::Max => p == &Piece::Max,
			Piece::Min => p == &Piece::Min,
		}
	}
}

impl Board {
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
			[[0, 1], [0, 2]],
			[[0, -1], [0, -2]],
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

				println!("TAKING FOR IDX {}", map_idx);

				self[&pos.clone().relocate(map[0][0], map[0][1]).unwrap()] = Piece::Empty;
				self[&pos.clone().relocate(map[1][0], map[1][1]).unwrap()] = Piece::Empty;
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

pub struct BoardIterator<'a> {
	board: &'a  Board,
	index: usize
}

impl<'a> IntoIterator for &'a Board {
    type Item = Position;
    type IntoIter = BoardIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BoardIterator {
            board: self,
            index: 0,
        }
    }
}


impl<'a> Iterator for BoardIterator<'a> {
	type Item = Position;

	fn next(&mut self) -> Option<Self::Item> {
		self.index += 1;

		if self.index >= 361 {
			return None;			
		}

		return Some(Position::from_u64(self.index));
	}
}