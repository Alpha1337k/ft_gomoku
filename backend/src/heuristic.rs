use std::{collections::{HashMap}, f32::INFINITY};

use crate::{board::Board, minimax::GameState, piece::{Piece, PieceWrap}, position::Position};

const DIRECTIONS: [[[i32; 2]; 2]; 4] = [
	[[-1, 0], [1, 0]], //x
	[[0, -1], [0, 1]], //y
	[[-1, -1], [1, 1]], //tlbr
	[[-1, 1], [1, -1]], //trbl
];

const CAPTURE_SCORES: [f32; 6] = [
	0.0,
	4.0,
	8.0,
	16.0,
	32.0,
	INFINITY
];

const B0_SCORES: [f32; 6] = [
	1.0,
	2.0,
	4.0,
	16.0,
	32.0,
	INFINITY
];

const B1_SCORES: [f32; 6] = [
	1.0,
	2.0,
	3.0,
	4.0,
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

#[derive(Clone)]
struct Line {
	id: usize,
	start: Position,
	end: Position,
	length: usize,
	direction: u8,
	score: f32,
	block_pos: u8,
	player: Piece
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
			score: Self::calculate(blocks, length, player)
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
	board: &'a Board,
	captures: &'a [usize; 2],
	lines: HashMap<usize, Line>,
	lines_idx: usize,
	line_pos: HashMap<Position, [usize; 4]>,
	score: Option<f32>
}

impl Heuristic<'_> {
	pub fn from_game_state(state: &GameState) -> Heuristic {
		Heuristic {
			lines_idx: 1,
			board: &state.board,
			captures: &state.captures,
			lines: HashMap::with_capacity(1),
			line_pos: HashMap::new(),
			score: None,
		}
	}

	pub fn from_board(board: &Board) -> Heuristic {
		Heuristic {
			lines_idx: 1,
			board: &board,
			captures: &[0, 0],
			lines: HashMap::with_capacity(1),
			line_pos: HashMap::new(),
			score: None,
		}
	}
	
	fn calculate_captures(value: &usize) -> f32 {
		match value {
			0..=5 => CAPTURE_SCORES[*value],
			_ => panic!()
		}
	}

	fn get_line(&self, pos: &Position, direction_idx: usize) -> Option<&Line> {

		let lines = self.line_pos.get(pos);
		if lines.is_none() {
			return None;
		} 
		
		let line = self.lines.get(&lines?[direction_idx]);

		return line;
	}

	fn get_position_score(pos: Position) -> f32 {
		let y = 1f32 - ((9.5f32 - (pos.y as f32)).abs() / 9.5f32);
		let x = 1f32 - ((9.5f32 - (pos.x % 19) as f32).abs() / 9.5f32);

		return (y + x) / 2f32;
	}

	fn get_position_scores(&self) -> f32 {
		let mut score = 0.0;
		
		for y in 0..19 {
			for x in 0..19 {
				let pos = Position::new(x, y);
				if self.board[&pos].is_max() {
					score += Self::get_position_score(pos);
				} else if self.board[&pos].is_min() {
					score -= Self::get_position_score(pos);
				}
			}
		}

		return score;
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

	fn populate_line_pos(&mut self, start: &Position, end: &Position, direction: [i32; 2], direction_idx: usize, reference_idx: usize)
	{
		let mut pos = start.clone();

		loop {
			// println!("POS: {} {}", pos, end);
			
			let p;
			
			if self.line_pos.contains_key(&pos) {
				p = self.line_pos.get_mut(&pos).unwrap();
			} else {
				self.line_pos.insert(pos, [0;4]);

				p = self.line_pos.get_mut(&pos).unwrap();
			}

			// println!("B4: {} {} {}", pos, p[direction_idx], reference_idx);
			
			p[direction_idx] = reference_idx;

			if pos == *end || pos.relocate(direction[0], direction[1]).is_err() {
				break;
			}
		}
	}

	fn evaluate_position(&mut self, pos: Position) {
		let directions = [
			[[-1, 0], [1, 0]], //x
			[[0, -1], [0, 1]], //y
			[[-1, -1], [1, 1]], //tlbr
			[[-1, 1], [1, -1]], //trbl
		];

		for (i, direction) in directions.iter().enumerate() {
			if self.get_line(&pos, i).is_some() {
				// println!("get_line cached already");
				continue;
			}

			let scores = [
				self.get_line_length(direction[0], pos, self.board[&pos]),
				self.get_line_length(direction[1], pos, self.board[&pos])
			];

			let block_count: u8 = ((scores[0].blocked as u8) << 1) + scores[1].blocked as u8;
			let length = 1 + scores[0].length + scores[1].length;

			// println!("SCORES: {} {} {} {} {}", pos, scores[0].length, scores[1].length, length, block_count);

			if length == 1 {
				// println!("continuing..");
				continue;
			}


			self.lines_idx += 1;

			self.lines.insert(self.lines_idx, 
				Line::new(self.lines_idx, self.board[&pos], block_count, scores[0].end, scores[1].end, i as u8, length)
			);
	
			let created_line = self.lines.get(&self.lines_idx).unwrap();

			self.populate_line_pos(
				&scores[0].end, 
				&scores[1].end, 
				direction[1], 
				i, 
				created_line.id
			);
		}
	}

	fn evaluate_positions(&mut self) {
		for pos in self.board.into_iter() {
			if self.board[&pos].is_piece() {
				self.evaluate_position(pos);
			}
		}
	}

	pub fn get_heuristic(&mut self) -> f32 {
		let mut scores = [0.0, 0.0];
		let capture_scores = [
			Self::calculate_captures(&self.captures[Piece::Max as usize]), 
			Self::calculate_captures(&self.captures[Piece::Min as usize]), 	
		];
	
		self.evaluate_positions();

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

		println!("{} {} {} {} {}", scores[0], scores[1], solo_scores, capture_scores[0], capture_scores[1]);

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

	pub fn evaluate_virtual_move(&self, pos: Position, player: Piece) -> Result<(f32, u8), &str> {

		let mut evaluation = self.score.unwrap();
		let mut captures = 0u8;

		// println!("LINES: {}", self.lines.len());

		for (i, direction) in DIRECTIONS.iter().enumerate() {
			let mut _nb_0 = pos.clone();
			let mut _nb_1 = pos.clone();

			let neighbor_lines = [
				 if _nb_0.relocate(direction[0][0], direction[0][1]).is_ok() {self.get_line(&_nb_0, i)} else {None},
				 if _nb_1.relocate(direction[1][0], direction[1][1]).is_ok() {self.get_line(&_nb_1, i)} else {None},
			];

			// println!("NB: {} {}", _nb_0, _nb_1);
			// println!("NL: {} {}", if neighbor_lines[0].is_some() {neighbor_lines[0].unwrap().length} else {1234},
			// 	if neighbor_lines[1].is_some() {neighbor_lines[1].unwrap().length} else {1234}
			// );

			let capture_map = [
				neighbor_lines[0].is_some_and(|x| x.player.is_opposite(&player) && neighbor_lines[0].unwrap().length == 2 && x.block_pos & 0x2 != 0),
				neighbor_lines[1].is_some_and(|x| x.player.is_opposite(&player) && neighbor_lines[1].unwrap().length == 2 && x.block_pos & 0x1 != 0)
			];

			let _block_map = [
				neighbor_lines[0].is_some_and(|x| x.player.is_opposite(&player)),
				neighbor_lines[1].is_some_and(|x| x.player.is_opposite(&player))				
			];

			let mut new_calc = 0.0;
			let mut blocks = 0;
			let mut length = 1;

			if neighbor_lines[0].is_some() && neighbor_lines[0].unwrap().player == player {
				blocks |= neighbor_lines[0].unwrap().block_pos & 0x2;
				length += neighbor_lines[0].unwrap().length;
				new_calc -= neighbor_lines[0].unwrap().score;
			}

			if neighbor_lines[1].is_some() && neighbor_lines[1].unwrap().player == player {
				blocks |= neighbor_lines[1].unwrap().block_pos & 0x1;
				length += neighbor_lines[1].unwrap().length;
				new_calc -= neighbor_lines[1].unwrap().score;
			}

			if neighbor_lines[0].is_some() && neighbor_lines[0].unwrap().player != player {
				let new_n_score = Line::calculate(neighbor_lines[0].unwrap().block_pos & 0x2 | 0x1, neighbor_lines[0].unwrap().length, player);
				new_calc -= neighbor_lines[0].unwrap().score;
				new_calc += new_n_score;
			}

			if neighbor_lines[1].is_some() && neighbor_lines[1].unwrap().player != player {
				let new_n_score = Line::calculate(neighbor_lines[1].unwrap().block_pos & 0x1 | 0x2, neighbor_lines[1].unwrap().length, player);
				new_calc -= neighbor_lines[1].unwrap().score;
				new_calc += new_n_score;
			}

			new_calc += Line::calculate(blocks, length, player);
		
			if capture_map[0] {
				captures |= 1u8 << (i * 2);
			} else if capture_map[1] {
				captures |= 1u8 << (i * 2) + 1;
			}


			evaluation += new_calc;
		}

		return Ok((evaluation, captures));
	}

	pub fn get_moves(&self, player: Piece) -> Vec<(Position, (f32, u8))> {
		let mut moves = HashMap::<Position, (f32, u8)>::with_capacity(50);
		
		for pos in self.board.into_iter() {
			if self.board[&pos].is_empty() {
				continue;
			}

			for y in -1..2 {
				for x in -1..2 {
					let mut check_pos = pos.clone();

					if (x == 0 && y == 0) || 
						check_pos.relocate(x, y).is_err() ||
						self.board[&check_pos].is_piece()
					{
						continue;
					}

					
					// check_pos = Position::new(5, 7);
					// println!("--- MOVE {} ---", check_pos);
					// if (self.validate_virtual_move(check_pos, player) == false) {
					// 	println!("INVALIDATED");
					// 	// return vec![];
					// 	continue;
					// }

					let mut eval = self.evaluate_virtual_move(check_pos, player).unwrap();

					let pos_score = Self::get_position_score(check_pos) / 4.0;

					if player == Piece::Max {
						eval.0 += pos_score;
					} else {
						eval.0 -= pos_score;
					}

					// println!("--- RESULT=MOVE {} Score={} ({})", check_pos, eval.0, eval.1);

					moves.insert(check_pos, eval);
				}
			}
		}

		let mut arr: Vec<(Position, (f32, u8))> = moves.into_iter().map(|f| (f.0, f.1)).collect();

		if player.is_max() {
			arr.sort_by(|a, b| b.1.0.total_cmp(&a.1.0));
		} else {
			arr.sort_by(|a, b| a.1.0.total_cmp(&b.1.0));
		}

		// for m in &arr {
		// 	println!("L: {} {:#010b}", m.0, m.1.1);
		// }
		return arr;
	}
}
