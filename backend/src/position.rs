use std::fmt;

use serde::{Deserialize, Serialize};



#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
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

	pub fn check_pos(&self) -> Result<&Position, &str> {
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
		if self.y >= 19 || self.x >= 19 {
			write!(f, "Invalid Position")
		} else {
			let y_char = (self.y as u8) + 65;
			write!(f, "{}{}", y_char as char, self.x)
		}

    }
}