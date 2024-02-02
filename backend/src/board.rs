use std::{fmt::{self, write}, ops::Add};

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
	data: Vec<Piece>
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
		if (self.x >= 19 || self.y >= 19) {
			return Err("Invalid position");
		}
		return Ok(self);
	}

	pub fn relocate(&mut self, d_x: i32, d_y: i32) -> Result<&Self, &str> {
		self.x = self.x.wrapping_add(d_x as usize);
		self.y = self.y.wrapping_add(d_y as usize);

		return self.check_pos();
	}

	pub fn to_u64(&self) -> u64 {
		return (self.y * 19 + self.x) as u64;
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
			data: vec![Piece::Empty; 19 * 19]
		 };

		 for (key, value) in board_map {
			let key_int = key.parse::<usize>().unwrap();
			board.data[key_int] = value.as_u64().unwrap().try_into().unwrap_or_default();
		}

		return board;
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
