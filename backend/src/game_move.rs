use crate::{heuristic::EvaluationScore, position::Position};

pub fn print_moveset(base_pos: &Position, base_order_idx: usize, base_score: f32, m: &Move) {
	let mut iter = m;

	print!(
		"Start score: {}, moves: ( {} ({}) [{}, {}] = {} )",
		base_score, base_pos, base_order_idx, iter.captures[0], iter.captures[1], iter.depth_score
	);

	loop {
		if iter.child.is_none() {
			break;
		}
		print!(" -> ");
		print!(
			"( {} ({}) [{}, {}]",
			iter.position, iter.order_idx, iter.captures[0], iter.captures[1]
		);

		if iter.child.is_some() {
			iter = iter.child.as_ref().unwrap().as_ref();
			print!(" = {})", iter.depth_score);
		}
	}
	println!();
}

#[derive(Debug)]
pub struct Move {
	pub score: f32,
	pub depth_score: f32,
	pub depth_hit: usize,
	pub position: Position,
	pub capture_map: u8,
	pub captures: [usize; 2],
	pub order_idx: usize,
	pub cutoff_at: usize,
	pub child: Option<Box<Move>>,
}

impl Default for Move {
	fn default() -> Self {
		Move {
			child: None,
			cutoff_at: 0,
			score: 0.0,
			depth_score: 0.0,
			depth_hit: 0,
			order_idx: 0,
			capture_map: 0,
			captures: [0, 0],
			position: Position::new(0, 0),
		}
	}
}

impl Move {
	pub fn update(
		&mut self,
		order: usize,
		child_move: Move,
		pos_move: &(Position, EvaluationScore),
	) {
		self.score = child_move.score;
		self.position = pos_move.0;
		self.order_idx = order;
		self.capture_map = pos_move.1.capture_map;
		self.depth_hit = child_move.depth_hit;
		self.child = Some(Box::new(child_move));
	}
}
