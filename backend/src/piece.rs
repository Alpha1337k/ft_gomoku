use std::{fmt};
use serde_repr::*;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PieceError {
    #[error("Invalid piece value: {value}")]
    InvalidValue { value: u64 },
}

#[derive(Clone, Copy, PartialEq, Deserialize_repr, Serialize_repr, Debug, Default)]
#[repr(i8)]
pub enum Piece {
	#[default]
	Empty = -1,
	Max = 0,
	Min = 1
}

pub trait PieceWrap {
	fn is_max(&self) -> bool;
	fn is_min(&self) -> bool;
	fn is_piece(&self) -> bool;
	fn is_empty(&self) -> bool;
	fn is_opposite(&self, p: &Piece) -> bool;
	fn is_equal(&self, p: &Piece) -> bool;

	fn get_opposite(&self) -> Piece;
}

impl TryFrom<u64> for Piece {
	type Error = PieceError;

	fn try_from(value: u64) -> Result<Self, Self::Error> {
		match value {
			x if x == Piece::Max as u64 => Ok(Piece::Max),
			x if x == Piece::Min as u64 => Ok(Piece::Min),
			_ => Err(PieceError::InvalidValue { value })
		}
	}
}

impl fmt::Display for Piece {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let to_write = match self {
			Piece::Empty => ' ',
			Piece::Max => 'X',
			Piece::Min => 'I',
		};
		write!(f, "{}", to_write)
	}
}

impl PieceWrap for Piece {
	fn is_max(&self) -> bool {
		matches!(self, Piece::Max)
	}
	fn is_min(&self) -> bool {
		matches!(self, Piece::Min)
	}
	fn is_piece(&self) -> bool {
		!matches!(self, Piece::Empty)
	}
	fn is_empty(&self) -> bool {
		matches!(self, Piece::Empty)
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

	fn get_opposite(&self) -> Piece {
		match self {
			Piece::Empty => panic!(),
			Piece::Max => Piece::Min,
			Piece::Min => Piece::Max
		}
	}
}
