use std::{cmp::Ordering, f32::INFINITY};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{board::Board, minimax::GameState, piece::{Piece, PieceWrap}, position::Position};

const DIRECTIONS: [[[i32; 2]; 2]; 4] = [
	[[-1, 0], [1, 0]], //x
	[[0, -1], [0, 1]], //y
	[[-1, -1], [1, 1]], //tlbr
	[[-1, 1], [1, -1]], //trbl
];

const CAPTURE_SCORES: [f32; 6] = [
	0.0,
	8.0,
	16.0,
	32.0,
	64.0,
	INFINITY
];

const B0_SCORES: [f32; 6] = [
	1.0,
	2.0,
	4.0,
	16.0,
	64.0,
	INFINITY
];

const B1_SCORES: [f32; 6] = [
	1.0,
	2.0,
	0.0, // can capture
	8.0,
	16.0,
	INFINITY
];

const B2_SCORES: [f32; 6] = [
	0.0,
	0.0,
	0.0,
	0.0,
	0.0,
	INFINITY
];
struct LineResult {
	end: Position,
	length: usize,
	blocked: bool,
}

pub struct EvaluationScore {
	pub score: f32,
	pub capture_map: u8,
	pub capture_count: usize
}

fn get_distance(p1: &Position, p2: &Position) -> i32
{
	((p1.x as f32 - p2.x as f32).powi(2) + (p1.y as f32 - p2.y as f32).powi(2)).sqrt() as i32
}

fn is_point_on_line(start: Position, end: Position, check: Position) -> bool {

	get_distance(&start, &end) == get_distance(&end, &check) + get_distance(&start, &check)
}


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

impl Line {
	pub fn new(idx: usize, player: Piece, blocks: u8, start: Position, end: Position, direction: u8, length: usize) -> Line {
		Line {
			id: idx,
			start: start,
			end: end,
			player: player,
			block_pos: blocks,
			direction: direction,
			length: length,
			score: Self::calculate(blocks, length, player),
		}
	}

	pub fn calculate(blocks: u8, length: usize, player: Piece) -> f32 {
		let val = match blocks {
			0 => B0_SCORES[length.min(5)],
			1 => B1_SCORES[length.min(5)],
			2 => B1_SCORES[length.min(5)],
			3 => B2_SCORES[length.min(5)],
			_ => INFINITY
		};

		val * if player == Piece::Max {1.0} else {-1.0}
	}
}

#[derive(Clone)]
pub struct Heuristic<'a> {
	pub board: &'a Board,
	pub captures: &'a [usize; 2],
	pub lines: FxHashMap<usize, Line>,
	pub lines_idx: usize,
	pub line_pos: FxHashMap<Position, [usize; 4]>,
	pub score: Option<f32>
}

impl Heuristic<'_> {
	pub fn from_game_state(state: &GameState) -> Heuristic {
		let mut h = Heuristic {
			lines_idx: 1,
			board: &state.board,
			captures: &state.captures,
			lines: FxHashMap::default(),
			line_pos: FxHashMap::default(),
			score: None,
		};

		for pos in h.board.into_iter() {
			if h.board[&pos].is_piece() {
				for (i, direction) in DIRECTIONS.iter().enumerate() {
					h.evaluate_position(pos, direction, i);
				}
			}
		}

		h
	}

	pub fn from_new_state<'a>(&'a self, state: &'a GameState) -> Heuristic {

		let mut n = Heuristic {
			lines_idx: self.lines_idx,
			board: &state.board,
			captures: &state.captures,
			lines: self.lines.clone(),
			line_pos: self.line_pos.clone(),
			score: None,
		};
	
		let diff = Board::get_diff(n.board, &self.board);

		let mut lines_to_delete = FxHashSet::<usize>::default();

		lines_to_delete.reserve(4);

		for pos in diff {
			for (i, direction) in DIRECTIONS.iter().enumerate() {
				let mut cur_poses = [
					pos.clone(),
					pos.clone(),
				];

				let mut needs_line_one_eval = true;
				let mut needs_line_two_eval = true;

				if cur_poses[0].relocate(direction[0][0], direction[0][1]).is_ok() {
					if let Some(l) = n.get_line_mut(&cur_poses[0], i) {
						lines_to_delete.insert(l.id);
					}
				} else {
					needs_line_one_eval = false;
				}
				
				if cur_poses[1].relocate(direction[1][0], direction[1][1]).is_ok() {
					if let Some(l) = n.get_line_mut(&cur_poses[1], i) {
						lines_to_delete.insert(l.id);
					}
				} else {
					needs_line_two_eval = false;
				}

				if needs_line_one_eval && n.board[&cur_poses[0]].is_piece() {
					let recalc = n.evaluate_position(cur_poses[0], direction, i);

					if recalc.2.is_some() {
						lines_to_delete.extend(recalc.2.unwrap());
					}
	
					if needs_line_two_eval == false || is_point_on_line(recalc.0, recalc.1, cur_poses[1]) {
						continue;
					}
				}
				if needs_line_two_eval && n.board[&cur_poses[1]].is_piece() {
					let recalc = n.evaluate_position(cur_poses[1], direction, i);
				
					if recalc.2.is_some() {
						lines_to_delete.extend(recalc.2.unwrap());
					}
				}
			}
		}

		for line in lines_to_delete {
			n.lines.remove(&line);
		}

		return n;
	}


	pub fn from_board<'a>(board: &'a Board, captures: &'a [usize; 2]) -> Heuristic<'a> {

		let mut h = Heuristic {
			lines_idx: 1,
			board: &board,
			captures: &captures,
			lines: FxHashMap::default(),
			line_pos: FxHashMap::default(),
			score: None,
		};

		for pos in h.board.into_iter() {
			if h.board[&pos].is_piece() {
				for (i, direction) in DIRECTIONS.iter().enumerate() {
					h.evaluate_position(pos, direction, i);
				}
			}
		}

		h
	}
	
	fn calculate_captures(value: &usize) -> f32 {
		CAPTURE_SCORES[(*value).min(5)]
	}

	fn get_line(&self, pos: &Position, direction_idx: usize) -> Option<&Line> {
		let lines = self.line_pos.get(pos);
		if lines.is_none() {
			return None;
		} 
		
		let line = self.lines.get(&lines?[direction_idx]);

		return line;
	}

	fn get_line_mut(&mut self, pos: &Position, direction_idx: usize) -> Option<&mut Line> {

		let lines = self.line_pos.get(pos);
		if lines.is_none() {
			return None;
		} 
		
		let line = self.lines.get_mut(&lines?[direction_idx]);

		return line;
	}

	fn get_position_score(pos: Position) -> f32 {
		let y = 1f32 - ((9.5f32 - (pos.y as f32)).abs() / 9.5f32);
		let x = 1f32 - ((9.5f32 - (pos.x % 19) as f32).abs() / 9.5f32);

		return (y + x) / 2f32;
	}

	fn get_line_length(&self, direction: [i32; 2], start: Position, player: Piece) -> LineResult
	{
		let mut pos = start.clone();

		let mut response = LineResult {
			blocked: true,
			end: start.clone(),
			length: 0
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

	fn populate_line_pos(&mut self, start: &Position, end: &Position, direction: [i32; 2], direction_idx: usize, reference_idx: usize) -> FxHashSet<usize>
	{
		let mut pos = start.clone();
		let mut overwritten_lines = FxHashSet::default();
		
		overwritten_lines.reserve(4);

		loop {
			let p;
			
			if self.line_pos.contains_key(&pos) {
				p = self.line_pos.get_mut(&pos).unwrap();
			} else {
				self.line_pos.insert(pos, [0;4]);

				p = self.line_pos.get_mut(&pos).unwrap();
			}

			if p[direction_idx] != 0 {
				overwritten_lines.insert(p[direction_idx]);
			}
			
			p[direction_idx] = reference_idx;

			if pos == *end || pos.relocate(direction[0], direction[1]).is_err() {
				break;
			}

		}
		return overwritten_lines;
	}

	fn evaluate_position(&mut self, pos: Position, direction: &[[i32; 2]; 2], direction_idx: usize) -> (Position, Position, Option<FxHashSet<usize>>) {
		let scores = [
			self.get_line_length(direction[0], pos, self.board[&pos]),
			self.get_line_length(direction[1], pos, self.board[&pos])
		];

		let block_count: u8 = ((scores[0].blocked as u8) << 1) + scores[1].blocked as u8;
		let length = 1 + scores[0].length + scores[1].length;

		// println!("SCORES: {} {} {} {} {}", pos, scores[0].length, scores[1].length, length, block_count);

		if length == 1 {
			// println!("continuing..");
			return (scores[0].end, scores[1].end, None);
		}


		self.lines_idx += 1;

		self.lines.insert(self.lines_idx, 
			Line::new(self.lines_idx, self.board[&pos], block_count, scores[0].end, scores[1].end, direction_idx as u8, length)
		);

		let created_line = self.lines.get(&self.lines_idx).unwrap();

		let overwritten = self.populate_line_pos(
			&scores[0].end, 
			&scores[1].end, 
			direction[1], 
			direction_idx, 
			created_line.id
		);

		return (scores[0].end, scores[1].end, Some(overwritten));
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

		for (_idx, line) in &self.lines {
			if line.player == Piece::Max {
				scores[0] += line.score;
			} else {
				scores[1] += line.score;
			}
		}


		let mut solo_scores = 0.0;

		for pos in self.board {
			match self.board[&pos] {
				Piece::Max => solo_scores += Self::get_position_score(pos),
				Piece::Min => solo_scores -= Self::get_position_score(pos),
				_ => ()
			}
		}

		// println!("{} {} {} {} {}", scores[0], scores[1], solo_scores, capture_scores[0], capture_scores[1]);

		self.score = Some(
			scores[0] + scores[1] + 
			solo_scores + 
			capture_scores[0] - capture_scores[1]
		);

		// for line in &self.lines {
		// 	println!("LN: {} {} L:{} S:{} B:{}", line.1.start, line.1.end, line.1.length, line.1.score, line.1.block_pos);
		// }

		return self.score.unwrap();
	}

	pub fn evaluate_virtual_move(&self, pos: Position, player: Piece) -> Result<EvaluationScore, &str> {
		let mut result = EvaluationScore {
			score: self.score.unwrap(),
			capture_map: 0u8,
			capture_count: 0,
		};

		// println!("LINES: {}", self.lines.len());

		for (i, direction) in DIRECTIONS.iter().enumerate() {
			let mut _nb_0 = pos.clone();
			let mut _nb_1 = pos.clone();

			let neighbor_lines = [
				 if _nb_0.relocate(direction[0][0], direction[0][1]).is_ok() {self.get_line(&_nb_0, i)} else {None},
				 if _nb_1.relocate(direction[1][0], direction[1][1]).is_ok() {self.get_line(&_nb_1, i)} else {None},
			];

			let neighbor_blocks = [
				if neighbor_lines[0].is_none() && _nb_0.check_pos().is_ok() {self.board[&_nb_0]} else {Piece::Empty},
				if neighbor_lines[1].is_none() && _nb_1.check_pos().is_ok() {self.board[&_nb_1]} else {Piece::Empty},
			];

			let capture_map = [
				neighbor_lines[0].is_some_and(|x| x.player.is_opposite(&player) && neighbor_lines[0].unwrap().length == 2 && x.block_pos & 0x2 != 0),
				neighbor_lines[1].is_some_and(|x| x.player.is_opposite(&player) && neighbor_lines[1].unwrap().length == 2 && x.block_pos & 0x1 != 0)
			];

			let _block_map = [
				neighbor_lines[0].is_some_and(|x| x.player.is_opposite(&player)),
				neighbor_lines[1].is_some_and(|x| x.player.is_opposite(&player))				
			];

			// println!("NB_L: {}, {} NB_B: {}, {} POS: {}", neighbor_lines[0].is_some(), neighbor_lines[1].is_some(), neighbor_blocks[0], neighbor_blocks[1], pos);

			let mut new_calc = 0.0;
			let mut blocks = 0;
			let mut length = 1;

			if neighbor_lines[0].is_some() && neighbor_lines[0].unwrap().player == player {
				blocks |= neighbor_lines[0].unwrap().block_pos & 0x2;
				length += neighbor_lines[0].unwrap().length;
				new_calc -= neighbor_lines[0].unwrap().score;
			} else if neighbor_blocks[0] == player {
				if _nb_0.relocate(direction[0][0], direction[0][1]).is_ok() && self.board[&_nb_0] == player.get_opposite() {
					blocks |= 0x2;
				}
				length += 1;
			}

			if neighbor_lines[1].is_some() && neighbor_lines[1].unwrap().player == player {
				blocks |= neighbor_lines[1].unwrap().block_pos & 0x1;
				length += neighbor_lines[1].unwrap().length;
				new_calc -= neighbor_lines[1].unwrap().score;
			} else if neighbor_blocks[1] == player {
				if _nb_1.relocate(direction[1][0], direction[1][1]).is_ok() && self.board[&_nb_1] == player.get_opposite() {
					blocks |= 0x1;
				}
				length += 1;
			}

			if neighbor_lines[0].is_some() && neighbor_lines[0].unwrap().player != player {
				let neighbor_blocks = (neighbor_lines[0].unwrap().block_pos & 0x2) | 0x1;

				blocks |= 0x2;

				let new_n_score = Line::calculate(neighbor_blocks, neighbor_lines[0].unwrap().length, player.get_opposite());
				new_calc -= neighbor_lines[0].unwrap().score;
				new_calc += new_n_score;
			} else if neighbor_blocks[0] == player.get_opposite() {
				blocks |= 0x2;
			}

			if neighbor_lines[1].is_some() && neighbor_lines[1].unwrap().player != player {
				let neighbor_blocks = (neighbor_lines[1].unwrap().block_pos & 0x1) | 0x2;

				blocks |= 0x1;

				let new_n_score = Line::calculate(neighbor_blocks, neighbor_lines[1].unwrap().length, player.get_opposite());
				new_calc -= neighbor_lines[1].unwrap().score;
				new_calc += new_n_score;
			} else if neighbor_blocks[1] == player.get_opposite() {
				blocks |= 0x1;
			}

			new_calc += Line::calculate(blocks, length, player);
		
			if capture_map[0] {
				result.capture_map |= 1u8 << (i * 2);
				result.capture_count += 1;
			} else if capture_map[1] {
				result.capture_map |= 1u8 << (i * 2) + 1;
				result.capture_count += 1;
			}


			result.score += new_calc;
		}

		return Ok(result);
	}

	fn sort_moves(mut arr: Vec<(Position, EvaluationScore)>, player: Piece) -> Vec<(Position, EvaluationScore)> {
		arr.sort_by(|a, b| {
			if a.1.capture_count > b.1.capture_count {
				return Ordering::Less;
			} else if a.1.capture_count < b.1.capture_count {
				return Ordering::Greater;
			}

			if player == Piece::Max {
				return b.1.score.total_cmp(&a.1.score);
			} else {
				return a.1.score.total_cmp(&b.1.score);
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
					let mut check_pos = pos.clone();

					if (x == 0 && y == 0) || 
						check_pos.relocate(x, y).is_err() ||
						self.board[&check_pos].is_piece() ||
						moves.contains_key(&check_pos)
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

					// println!("--- RESULT=MOVE {} Score={} ({})", check_pos, eval.0, eval.1);

					moves.insert(check_pos, eval);
				}
			}
		}

		let arr: Vec<(Position, EvaluationScore)> = moves.into_iter().collect();


		// for m in &arr {
		// 	println!("L: {} {:#010b}", m.0, m.1.1);
		// }
		return Self::sort_moves(arr, player);
	}
}
