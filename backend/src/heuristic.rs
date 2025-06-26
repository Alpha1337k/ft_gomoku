use std::cmp::Ordering;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
	block_position::BlockPosition,
	board::Board,
	constants::{BOARD_DIM, CAPTURE_SCORES, DIRECTIONS},
	line::Line,
	line_store::LineStore,
	minimax::GameState,
	piece::{Piece, PieceWrap},
	position::Position,
};

#[derive(Clone, Debug)]
struct LineResult {
	end: Position,
	length: usize,
	blocked: bool,
}

pub struct EvaluationScore {
	pub score: f32,
	pub capture_map: u8,
	pub capture_count: usize,
}

fn get_distance(p1: &Position, p2: &Position) -> i32 {
	((p1.x as f32 - p2.x as f32).powi(2) + (p1.y as f32 - p2.y as f32).powi(2)).sqrt() as i32
}

fn is_point_on_line(start: Position, end: Position, check: Position) -> bool {
	get_distance(&start, &end) == get_distance(&end, &check) + get_distance(&start, &check)
}

#[derive(Clone)]
pub struct Heuristic {
	pub board: Board,
	pub captures: [usize; 2],
	pub lines: LineStore,
	pub score: Option<f32>,
}

impl Heuristic {
	pub fn new(board: &Board, captures: &[usize; 2]) -> Heuristic {
		let mut h = Heuristic {
			board: board.clone(),
			captures: *captures,
			lines: LineStore::default(),
			score: None,
		};

		h.evaluate();

		h
	}

	// Load heuristic from old heuristic, only recalculating the new move. This reduces calculations by a magnitude.
	pub fn from_old_state(&self, state: &GameState) -> Heuristic {
		let mut n = Heuristic {
			board: state.board.clone(),
			captures: state.captures,
			lines: self.lines.clone(),
			score: None,
		};

		let diff = Board::get_diff(&n.board, &self.board);

		let mut lines_to_delete = FxHashSet::<usize>::default();
		lines_to_delete.reserve(4);

		for pos in diff {
			for (i, direction) in DIRECTIONS.iter().enumerate() {
				if let Ok(pos) = pos.clone().relocate(direction[0][0], direction[0][1]) {
					if let Some(l) = n.lines.get_mut(pos, i) {
						lines_to_delete.insert(l.id);
					}

					if n.board[pos].is_piece() {
						let recalculation = n.evaluate_position(*pos, direction, i);

						if recalculation.2.is_some() {
							lines_to_delete.extend(recalculation.2.unwrap());
						}

						if pos
							.clone()
							.relocate(direction[1][0], direction[1][1])
							.is_ok_and(|pos| {
								is_point_on_line(recalculation.0, recalculation.1, *pos)
							}) {
							continue;
						}
					}
				}

				if let Ok(pos) = pos.clone().relocate(direction[1][0], direction[1][1]) {
					if let Some(l) = n.lines.get_mut(pos, i) {
						lines_to_delete.insert(l.id);
					}

					if n.board[pos].is_piece() {
						let recalculation = n.evaluate_position(*pos, direction, i);

						if recalculation.2.is_some() {
							lines_to_delete.extend(recalculation.2.unwrap());
						}
					}
				}
			}
		}

		for line in lines_to_delete {
			n.lines.remove(&line);
		}

		n
	}

	fn calculate_captures(value: &usize) -> f32 {
		CAPTURE_SCORES[(*value).min(5)]
	}

	fn get_position_score(pos: Position) -> f32 {
		let y = 1f32 - ((9.5f32 - (pos.y as f32)).abs() / 9.5f32);
		let x = 1f32 - ((9.5f32 - (pos.x % BOARD_DIM) as f32).abs() / 9.5f32);

		(y + x) / 2f32
	}

	fn get_line_length(&self, direction: [i32; 2], start: Position, player: Piece) -> LineResult {
		let mut pos = start;
		let mut response = LineResult {
			blocked: true,
			end: start,
			length: 0,
		};

		loop {
			if pos.relocate(direction[0], direction[1]).is_err() {
				return response;
			}

			if self.board[&pos].is_opposite(&player) {
				if self.board[&pos].is_empty() {
					response.blocked = false;
					return response;
				}
				return response;
			}

			response.length += 1;
			response.end = pos;
		}
	}

	fn evaluate_position(
		&mut self,
		pos: Position,
		direction: &[[i32; 2]; 2],
		direction_idx: usize,
	) -> (Position, Position, Option<FxHashSet<usize>>) {
		let scores = [
			self.get_line_length(direction[0], pos, self.board[&pos]),
			self.get_line_length(direction[1], pos, self.board[&pos]),
		];

		let block_count: u8 = ((scores[1].blocked as u8) << 1) | scores[0].blocked as u8;
		let length = 1 + scores[0].length + scores[1].length;

		if length == 1 {
			// println!("continuing..");
			return (scores[0].end, scores[1].end, None);
		}

		let overwritten = self.lines.insert(
			self.board[&pos],
			block_count.try_into().unwrap(),
			scores[0].end,
			scores[1].end,
			direction_idx,
			length,
		);

		(scores[0].end, scores[1].end, Some(overwritten))
	}

	pub fn get_heuristic(&mut self) -> f32 {
		let mut scores = [0.0, 0.0];
		let capture_scores = [
			Self::calculate_captures(&self.captures[Piece::Max as usize]),
			Self::calculate_captures(&self.captures[Piece::Min as usize]),
		];

		if capture_scores[0].is_infinite() || capture_scores[1].is_infinite() {
			self.score = Some(capture_scores[0] - capture_scores[1]);

			return self.score.unwrap();
		}

		for line in self.lines.values() {
			if line.player == Piece::Max {
				scores[0] += line.score;
			} else {
				scores[1] += line.score;
			}
		}

		let mut solo_scores = 0.0;

		for pos in &self.board {
			match self.board[&pos] {
				Piece::Max => solo_scores += Self::get_position_score(pos),
				Piece::Min => solo_scores -= Self::get_position_score(pos),
				_ => (),
			}
		}

		self.score =
			Some(scores[0] + scores[1] + solo_scores + capture_scores[0] - capture_scores[1]);

		self.score.unwrap()
	}

	pub fn evaluate(&mut self) -> &Self {
		let mut lines_to_delete = FxHashSet::<usize>::default();

		for pos in self.board.into_iter() {
			if self.board[&pos].is_piece() {
				for (i, direction) in DIRECTIONS.iter().enumerate() {
					let result = self.evaluate_position(pos, direction, i);

					if result.2.is_some() {
						lines_to_delete.extend(result.2.unwrap());
					}
				}
			}
		}

		for line in lines_to_delete {
			self.lines.remove(&line);
		}

		self
	}

	// Gets result of move without actually setting it.
	fn get_virtual_move_constraints(
		&self,
		player: Piece,
		block_value: BlockPosition,
		neighbor_line: Option<&Line>,
		neighbor: Piece,
		cursor: &Position,
		direction: [i32; 2],
	) -> (f32, BlockPosition, usize) {
		let mut delta = 0.0;
		let mut blocks = BlockPosition::NONE;
		let mut length = 0;

		let opposite_block = if block_value == BlockPosition::END {
			BlockPosition::START
		} else {
			BlockPosition::END
		};

		if let Some(line) = neighbor_line {
			if line.player != player {
				let neighbor_blocks = (line.block_pos & block_value) | opposite_block;

				let new_n_score =
					Line::calculate(neighbor_blocks, line.length, player.get_opposite());
				delta -= line.score;
				delta += new_n_score;
			} else {
				blocks |= line.block_pos & block_value;
				length += line.length;
				delta -= line.score;
			}
		}

		if neighbor == player.get_opposite() {
			blocks |= block_value;
		} else if neighbor == player {
			if cursor.clone().relocate(direction[0], direction[1]).is_ok()
				&& self.board[cursor] == player.get_opposite()
			{
				blocks |= block_value;
			}
			length += 1;
		}

		(delta, blocks, length)
	}

	// Gets result of moves without actually setting them.
	pub fn evaluate_virtual_move(
		&self,
		pos: Position,
		player: Piece,
	) -> Result<EvaluationScore, &str> {
		let mut result = EvaluationScore {
			score: self.score.unwrap(),
			capture_map: 0u8,
			capture_count: 0,
		};

		for (i, direction) in DIRECTIONS.iter().enumerate() {
			// Neighbor cursors.
			let mut _nb_0 = pos;
			let mut _nb_1 = pos;

			let neighbor_lines = [
				if _nb_0.relocate(direction[0][0], direction[0][1]).is_ok() {
					self.lines.get(&_nb_0, i)
				} else {
					None
				},
				if _nb_1.relocate(direction[1][0], direction[1][1]).is_ok() {
					self.lines.get(&_nb_1, i)
				} else {
					None
				},
			];

			let neighbor_blocks = [
				if neighbor_lines[0].is_none() && _nb_0.check_pos().is_ok() {
					self.board[&_nb_0]
				} else {
					Piece::Empty
				},
				if neighbor_lines[1].is_none() && _nb_1.check_pos().is_ok() {
					self.board[&_nb_1]
				} else {
					Piece::Empty
				},
			];

			let capture_map = [
				neighbor_lines[0].is_some_and(|x| {
					x.player.is_opposite(&player)
						&& neighbor_lines[0].unwrap().length == 2
						&& x.block_pos.has_start()
				}),
				neighbor_lines[1].is_some_and(|x| {
					x.player.is_opposite(&player)
						&& neighbor_lines[1].unwrap().length == 2
						&& x.block_pos.has_end()
				}),
			];

			// println!("NB_L: {}, {} NB_B: {}, {} POS: {}", neighbor_lines[0].is_some(), neighbor_lines[1].is_some(), neighbor_blocks[0], neighbor_blocks[1], pos);

			let mut new_calc = 0.0;
			let mut length = 1;
			let mut blocks = BlockPosition::NONE;

			let (d1_score, d1_blocks, d1_length) = self.get_virtual_move_constraints(
				player,
				BlockPosition::START,
				neighbor_lines[0],
				neighbor_blocks[0],
				&_nb_0,
				direction[0],
			);

			new_calc += d1_score;
			length += d1_length;
			blocks |= d1_blocks;

			let (d2_score, d2_blocks, d2_length) = self.get_virtual_move_constraints(
				player,
				BlockPosition::END,
				neighbor_lines[1],
				neighbor_blocks[1],
				&_nb_1,
				direction[1],
			);

			new_calc += d2_score;
			length += d2_length;
			blocks |= d2_blocks;

			new_calc += Line::calculate(blocks, length, player);

			if capture_map[0] {
				result.capture_map |= 1u8 << (i * 2);
				result.capture_count += 1;
			} else if capture_map[1] {
				result.capture_map |= 1u8 << ((i * 2) + 1);
				result.capture_count += 1;
			}

			result.score += new_calc;
		}

		Ok(result)
	}

	fn sort_moves(
		mut arr: Vec<(Position, EvaluationScore)>,
		player: Piece,
	) -> Vec<(Position, EvaluationScore)> {
		arr.sort_by(|a, b| {
			match a.1.capture_count.cmp(&b.1.capture_count) {
				Ordering::Greater => return Ordering::Less,
				Ordering::Less => return Ordering::Greater,
				Ordering::Equal => {}
			}

			if player == Piece::Max {
				b.1.score.total_cmp(&a.1.score)
			} else {
				a.1.score.total_cmp(&b.1.score)
			}
		});

		arr
	}

	pub fn get_moves(&self, player: Piece) -> Vec<(Position, EvaluationScore)> {
		let mut moves = FxHashMap::<Position, EvaluationScore>::default();

		moves.reserve(50);

		for pos in self.board.into_iter() {
			if self.board[&pos].is_empty() {
				continue;
			}

			for y in -1..2 {
				for x in -1..2 {
					let mut check_pos = pos;

					if (x == 0 && y == 0)
						|| check_pos.relocate(x, y).is_err()
						|| self.board[&check_pos].is_piece()
						|| moves.contains_key(&check_pos)
					{
						continue;
					}

					let mut eval = self.evaluate_virtual_move(check_pos, player).unwrap();

					let pos_score = Self::get_position_score(check_pos) / 4.0;

					if player == Piece::Max {
						eval.score += pos_score;
					} else {
						eval.score -= pos_score;
					}

					moves.insert(check_pos, eval);
				}
			}
		}

		let arr: Vec<(Position, EvaluationScore)> = moves.into_iter().collect();

		Self::sort_moves(arr, player)
	}
}
