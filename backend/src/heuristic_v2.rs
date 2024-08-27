use std::collections::HashMap;

use crate::{board::Board, piece::Piece, position::Position};

#[derive(Clone, Debug)]
pub struct Line {
	pub id: usize,
	pub start: Position,
	pub end: Position,
	pub length: usize,
	pub direction: u8,
	pub score: f32,
	pub block_pos: u8,
	pub player: Piece,
}

struct LineStore {
	pub lines: [Vec<Line>; 4],
}



#[derive(Clone)]
pub struct HeuristicNew<'a> {
	pub board: &'a Board,
	pub captures: &'a [usize; 2],
	pub lines: HashMap<usize, Line>,
	pub score: Option<f32>
}

/*

	0 0 = Line(2)

	9  9
	10 10
	11 11
	12 12

	10 9
	11 10

*/