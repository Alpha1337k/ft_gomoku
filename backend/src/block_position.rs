use std::ops::{BitAnd, BitOr, BitOrAssign};

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BlockPositionError {
	#[error("Position exceeds bounds: {value}")]
	ExceedsBounds { value: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPosition {
	NONE = 0x0,
	START = 0x1,
	END = 0x2,
	BOTH = 0x3,
}

impl BlockPosition {
	pub fn is_none(&self) -> bool {
		*self == BlockPosition::NONE
	}

	pub fn has_start(&self) -> bool {
		*self == BlockPosition::START
	}

	pub fn has_end(&self) -> bool {
		*self == BlockPosition::END
	}

	pub fn is_both(&self) -> bool {
		*self == BlockPosition::BOTH
	}
}

impl BitOr for BlockPosition {
	type Output = BlockPosition;

	fn bitor(self, other: Self) -> BlockPosition {
		(self as u8 | other as u8)
			.try_into()
			.unwrap_or(BlockPosition::NONE)
	}
}

impl BitOrAssign for BlockPosition {
	fn bitor_assign(&mut self, other: Self) {
		*self = (*self as u8 | other as u8)
			.try_into()
			.unwrap_or(BlockPosition::NONE);
	}
}

impl TryFrom<u8> for BlockPosition {
	type Error = BlockPositionError;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0x0 => Ok(BlockPosition::NONE),
			0x1 => Ok(BlockPosition::START),
			0x2 => Ok(BlockPosition::END),
			0x3 => Ok(BlockPosition::BOTH),
			_ => Err(BlockPositionError::ExceedsBounds { value }),
		}
	}
}

impl BitAnd for BlockPosition {
	type Output = BlockPosition;

	fn bitand(self, other: Self) -> BlockPosition {
		(self as u8 & other as u8)
			.try_into()
			.unwrap_or(BlockPosition::NONE)
	}
}
