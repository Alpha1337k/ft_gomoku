use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, f32::{INFINITY}, io::Error, net::{TcpStream}};

use serde_json::json;
use websocket::{sync::Writer};

use crate::{board::{Board, Piece, PieceWrap, Position}, WSMessage};

pub struct GomokuSolver<'a>
{
	pub board: Board,
	turn_idx: u8,
	sender: Option<&'a mut Writer<TcpStream>>,
	depth_zero_hits: usize
}

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: &'a mut Writer<TcpStream>) -> Result<GomokuSolver<'a>, Error> {
		let board_raw = msg.data.as_object().unwrap().get("board").unwrap().as_object().unwrap();

		let mut solver = GomokuSolver{
			board: Board::from_map(board_raw),
			turn_idx: msg.data.as_object().unwrap().get("currentTurn").unwrap_or(&json!(0)).as_i64().unwrap() as u8,
			sender: Some(sender),
			depth_zero_hits: 0,
		};

		return Ok(solver);
	}

	fn check_capture(d_x: i32, d_y: i32, mut pos: Position, board: &Board, player: Piece) -> bool
	{
		for i in 0..3 {
			if (pos.relocate(d_x, d_y).is_err()) {
				return false;
			}

			if (i < 2 && board[&pos].is_equal(&player)) {
				return false;
			}
			else if (i == 2 && board[&pos].is_opposite(&player)) {
				return false;
			}
		}

		return true;
	}

	fn evaluate_possible_move(board: &mut Board, mut pos: Position, is_maximizing: bool) -> Option<(f32, u8)> {
		let coords = [
			[[-1, 0], [1, 0]], //x
			[[0, 1], [0, -1]], //y
			[[-1, -1], [1, 1]], //tlbr
			[[-1, 1], [1, -1]], //trbl
		];

		let mut total_score = 0.0;
		let mut capture_directions = 0u8;

		if (is_maximizing) {
			board[&pos] = Piece::Max;
		} else {
			board[&pos] = Piece::Min;
		}

		for (i, direction) in coords.iter().enumerate() {
			let score_1 = Self::get_score(direction[0][0], direction[0][1], pos, board, None);
			let score_2 = Self::get_score(direction[1][0], direction[1][1], pos, board, None);
			let capture_1 = Self::check_capture(direction[0][0], direction[0][1], pos, board, if is_maximizing {Piece::Min} else {Piece::Max});
			let capture_2 = Self::check_capture(direction[1][0], direction[1][1], pos, board, if is_maximizing {Piece::Min} else {Piece::Max});


			let mut length = (score_1.0 + score_2.0 + 1) as usize;

			if (capture_1) {
				println!("CAPTURE1");
				capture_directions |= (1u8 << i);
				length *= 3;
			}
			if (capture_2) {
				println!("CAPTURE2");
				capture_directions |= (1u8 << i + 1);
				length *= 3;
			}

			println!("LENGTH: {} {}+{}+1", pos, score_1.0, score_2.0);

			if (length == 3 && score_1.1 == false && score_2.1 == false) {
				board[&pos] = Piece::Empty;
				return None;
			}

			total_score += (length.pow(2)) as f32;
		}

		board[&pos] = Piece::Empty;
		return Some((total_score, capture_directions));
	}

	pub fn get_possible_moves(board: &Board, is_maximizing: bool) -> Vec<(Position, u8)>
	{
		let mut position_set = HashMap::<Position, Option<(f32, u8)>>::with_capacity(64);
		let mut board_clone = board.clone();

		for y in 0..19 {
			for x in 0..19 {
				let base_pos = Position::new(x, y);
				if (board[&base_pos].is_empty()) {
					continue;
				}
				for d_y in -1i32..2 {
					for d_x in -1i32..2 {
						let mut off_pos = base_pos.clone();
						if ((d_x == 0 && d_y == 0) || off_pos.relocate(d_x, d_y).is_err()) {
							continue;
						}

						println!("PM: {}", off_pos);

						if board[&off_pos].is_empty() && position_set.contains_key(&off_pos) == false {
							let evaluation = Self::evaluate_possible_move(&mut board_clone, off_pos, is_maximizing);
							if (evaluation.is_none()) {
								position_set.insert(off_pos, None);
								continue;
							}
							position_set.insert(off_pos, Some((
								evaluation.unwrap().0 + Self::get_position_score(off_pos),
								evaluation.unwrap().1
								))
							);
						}
					}
				}
			}
		}

		let mut position_arr: Vec<(Position, (f32, u8))> = position_set.into_iter().filter(|x| x.1.is_some()).map(|f| (f.0, f.1.unwrap())).collect();

		position_arr.sort_by(|a, b| b.1.0.total_cmp(&a.1.0));
		position_arr.iter().for_each(|f| println!("{}: {} ({})", f.0, f.1.0, f.1.1));


		return position_arr.iter().map(|v| (v.0, v.1.1)).collect();
	}

	fn get_score(d_x: i32, d_y: i32, start: Position, board: &Board, mut visited_places: Option<&mut HashSet<Position>>) -> (u8, bool)
	{
		let mut pos = start.clone();
		let mut len = 0;

		loop {
			if pos.relocate(d_x, d_y).is_err() {
				return (len, true);
			}

			if board[&start].is_opposite(&board[&pos]) {
				if board[&pos].is_empty() {
					return (len, false);
				}
				return (len, true);
			}

			if (visited_places.is_some()) {
				visited_places.as_deref_mut().unwrap().insert(pos);
			}

			len += 1;
		}
	}

	fn get_surround_score(visited_places: &mut HashSet<Position>, start: Position, board: &Board) -> f32 {
		let coords = [
			[[-1, 0], [1, 0]], //x
			[[0, 1], [0, -1]], //y
			[[-1, -1], [1, 1]], //tlbr
			[[-1, 1], [1, -1]], //trbl
		];

		let mut total_score = 0.0;

		for (_i, direction) in coords.iter().enumerate() {
			let score_1 = Self::get_score(direction[0][0], direction[0][1], start, board, Some(visited_places));
			let score_2 = Self::get_score(direction[1][0], direction[1][1], start, board, Some(visited_places));

			let length: usize = (score_1.0 + score_2.0 + 1).into();
			let mut score: f32 = length as f32;

			if length >= 5 {
				return INFINITY;
			}

			score *= length as f32;

			if score_1.1 == false && score_2.1 == false {
				score *= 1.3;
			}

			else if score_1.1 == true && score_2.1 == true {
				score = 0.0;
			}

			total_score += score;
		}

		return total_score;
	}

	fn get_position_scores(board: &Board) -> f32 {
		let mut score = 0.0;
		
		for y in 0..19 {
			for x in 0..19 {
				let pos = Position::new(x, y);
				if board[&pos].is_max() {
					score += Self::get_position_score(pos);
				} else if board[&pos].is_min() {
					score -= Self::get_position_score(pos);
				}
			}
		}

		return score;
	}

	pub fn get_heuristic(board: &Board) -> f32 {
		let mut maximizing_score = 0.0;
		let mut minimizing_score = 0.0;
		let mut visited_places = HashSet::<Position>::with_capacity(19 * 19);

		for y in 0..19 {
			for x in 0..19 {
				let pos = Position::new(x, y);

				if minimizing_score == INFINITY || maximizing_score == INFINITY {
					return maximizing_score - minimizing_score;
				}

				if board[&pos].is_empty() || visited_places.get(&pos).is_some() {
					continue;
				}
				if board[&pos].is_max() {
					maximizing_score = Self::get_surround_score(&mut visited_places, pos, board);
				} else if board[&pos].is_min() {
					minimizing_score = Self::get_surround_score(&mut visited_places, pos, board);
				}
			}
		}

		let mut score: f32 = Self::get_position_scores(board);

		score += maximizing_score - minimizing_score;

		return score;
	}

	fn set_move(board: &mut Board, mut pos: Position, val: Piece, mut capture_map: u8)
	{
		board[&pos] = val;

		if (capture_map == 0) {
			return;
		}

		let maps = [
			[[-1, 0], [-2, 0]],
			[[1, 0], [2, 0]],
			[[0, 1], [0, 2]],
			[[0, -1], [0, -2]],
			[[-1, -1], [-2, -2]],
			[[1, 1], [2, 2]],
			[[-1, 1], [-2, 2]],
			[[1, 1], [2, -2]],
		];

		let mut map_idx = 0;
		while capture_map != 0 {
			let needs_capture = capture_map & 0x1;
			if (needs_capture == 1) {
				let map = maps[map_idx];

				board[&pos.clone().relocate(map[0][0], map[0][1]).unwrap()] = Piece::Empty;
				board[&pos.clone().relocate(map[1][0], map[1][1]).unwrap()] = Piece::Empty;
			}
			capture_map >>= 1;
			map_idx += 1;
		}
	}

	fn minimax(&mut self, depth: usize, board: &Board, mut alpha: f32, mut beta: f32, is_maximizing: bool) -> (f32, Vec<Position>)
	{
		let mut moves = Vec::<Position>::with_capacity(depth);
		let heuristic = Self::get_heuristic(board);

		println!("H: {}", heuristic);

		if depth == 0 || heuristic.is_infinite() {
			self.depth_zero_hits += 1;
			return (heuristic, moves);
		}

		let possible_moves = Self::get_possible_moves(board, is_maximizing);

		println!("MOVES: {}", possible_moves.len());

		moves.push(Position::new(0, 0));

		let _squares_checked = 0;

		if is_maximizing
		{
			let mut val = -INFINITY;

			for i in possible_moves {

				if depth >= 3 {
					println!("PGR @D {}: {} D0: {}", depth, i.0, self.depth_zero_hits);
				}

				let mut new_board = board.clone();

				Self::set_move(&mut new_board, i.0, Piece::Max, i.1);

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);

				println!("RES: pos: {} V:{}", i.0, node_result.0);

				if node_result.0 > val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i.0;
					moves.append(&mut node_result.1);
				}

				if val > alpha {
					alpha = val;
				}
				if val > beta {
					// println!("BETA BREAK {}", depth);
					break;
				}
			}
			return (val, moves);
		}
		else
		{
			let mut val = INFINITY;

			for i in possible_moves {
				if depth >= 3 {
					println!("PGR @D {}: {} D0: {}", depth, i.0, self.depth_zero_hits);
				}

				let mut new_board = board.clone();

				Self::set_move(&mut new_board, i.0, Piece::Min, i.1);


				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);

				println!("RES: pos: {} V:{}", i.0, node_result.0);

				if node_result.0 < val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i.0;
					moves.append(&mut node_result.1);
				}

				if val < beta {
					beta = val;
				}
				if val < alpha {
					// println!("ALPHA BREAK {}", depth);
					break;
				}
			}
			return (val, moves);
		}
	}

	pub fn solve<'a>(&mut self) -> Result<(f32, Vec<Position>), Error>
	{
		println!("Starting minimax..");

		let res = self.minimax(4, &self.board.clone(), -INFINITY, INFINITY, self.turn_idx % 2 == 0);

		for m in &res.1 {
			println!("M: {}", m);
		}
		println!("SCORE: {}", res.0);

		return Ok(res);
	}
}
