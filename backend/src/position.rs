use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PositionError {
    #[error("Position exceeds bounds: {value_x}, {value_y}")]
    ExceedsBounds { value_x: usize, value_y: usize },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Position {
	pub x: usize,
	pub y: usize,
}

impl Position {
	pub fn new(x: usize, y: usize) -> Position {
		*Position {
			x,
			y,
		}.check_pos().unwrap()
	}

	pub fn check_pos(&self) -> Result<&Position, PositionError> {
		if self.x >= 19 || self.y >= 19 {
			return Err(PositionError::ExceedsBounds { value_x: self.x, value_y: self.y });
		}
		Ok(self)
	}

	pub fn relocate(&mut self, d_x: i32, d_y: i32) -> Result<&Self, PositionError> {
		self.x = self.x.wrapping_add(d_x as usize);
		self.y = self.y.wrapping_add(d_y as usize);

		self.check_pos()
	}

	pub fn relocate_n(&mut self, d_x: i32, d_y: i32, n: usize) -> Result<&Self, PositionError> {
		for _i in 0..n {
			self.relocate(d_x, d_y)?;
		}

		Ok(self)
	}

	pub fn to_u64(&self) -> u64 {
		(self.y * 19 + self.x) as u64
	}

	pub fn from_u64(pos: usize) -> Position {
		*Position {
			x: pos % 19,
			y: pos.div_euclid(19)
		}.check_pos().unwrap()
	}
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.check_pos().is_err() {
			write!(f, "Invalid Position")
		} else {
			let y_char = (self.y as u8) + 65;
			write!(f, "{}{}", y_char as char, self.x)
		}
    }
}