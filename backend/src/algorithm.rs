use std::{cmp::{max, max_by}, collections::{HashMap, HashSet}, f32::INFINITY, io::Error, net::{SocketAddr, TcpStream}, thread::sleep, time::{Duration, SystemTime}};

use websocket::{sync::Writer, OwnedMessage};

use crate::{WSMessage};

pub struct GomokuSolver<'a>
{
	board: Vec<u8>,
	turn_idx: u8,
	sender: Option<&'a mut Writer<TcpStream>>,
}

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: &'a mut Writer<TcpStream>) -> Result<GomokuSolver<'a>, Error> {
		let board_raw = msg.data.as_object().unwrap().get("board").unwrap().as_object().unwrap();

		let mut solver = GomokuSolver{
			board: vec![u8::MAX; 19 * 19],
			turn_idx: 0,
			sender: Some(sender),
		};
		
		for (key, value) in board_raw {
			let key_int = key.parse::<usize>().unwrap();
			solver.board[key_int] = value.as_i64().unwrap() as u8;
		}

		return Ok(solver);
	}

	fn get_score(x: i32, y: i32, start_idx: usize, board: &Vec<u8>, visited_places: &mut HashSet<usize>) -> u8
	{
		let mut idx: i32 = start_idx as i32;
		let mut len = 0;
		loop {
			visited_places.insert(idx as usize);

			let old_idx = idx;
			idx += (y * 19 + x);

			if (
				idx >= 19 * 19 || 
				idx < 0 || 
				(old_idx % 19 == 18 && idx % 19 == 0) ||
				(old_idx % 19 == 0 && idx % 19 == 18) ||
				board[idx as usize] != board[start_idx]
			) {
				return len;
			}

			len += 1;
		}
	}

	fn get_surround_score(scores: &mut [i32], visited_places: &mut HashSet<usize>, start_idx: usize, board: &Vec<u8>) {

		// x, y, tlbr, trbl
		let directions = [
			Self::get_score(-1, 0, start_idx, board, visited_places) + Self::get_score(1, 0, start_idx, board, visited_places),
			Self::get_score(1, 0, start_idx, board, visited_places) + Self::get_score(-1, 0, start_idx, board, visited_places),
			Self::get_score(-1, -1, start_idx, board, visited_places) + Self::get_score(1, 1, start_idx, board, visited_places),
			Self::get_score(-1, 1, start_idx, board, visited_places) + Self::get_score(1, -1, start_idx, board, visited_places),
		];

		scores[0] += 1;

		for direction in directions {
			if (direction == 0) {
				continue;
			}
			scores[direction as usize] += 1;
		}
	}

	fn get_position_score(pos: usize) -> f32 {
		let y = 1f32 - ((9.5 - (pos as f32 / 19f32).floor()).abs() / 9.5f32);
		let x = 1f32 - ((9.5 - (pos % 19) as f32).abs() / 9.5f32);

		return (y + x) / 4f32;
	}

	fn get_position_scores(board: &Vec<u8>) -> f32 {
		let mut score = 0.0;
		
		for i in 0..board.len() {
			if (board[i] == 0) {
				score += Self::get_position_score(i);
			} else if (board[i] == 1) {
				score -= Self::get_position_score(i);
			}
		}

		return score;
	}

	fn get_heuristic(board: &Vec<u8>) -> f32 {
		let mut p0_scores = [0, 0, 0, 0, 0];
		let mut p1_scores = [0, 0, 0, 0, 0];
		let mut visited_places = HashSet::<usize>::with_capacity(19 * 19);

		for i in 0..board.len() {
			if (board[i] == u8::MAX || visited_places.get(&i).is_some()) {
				continue;
			}

			if (board[i] == 0) {
				Self::get_surround_score(&mut p0_scores, &mut visited_places, i, board);
			} else if (board[i] == 1) {
				Self::get_surround_score(&mut p1_scores, &mut visited_places, i, board);
			}
		}

		let mut score: f32 = Self::get_position_scores(board);

		for i in 0..5 {
			score += ((p0_scores[i] - p1_scores[i]) as i64 * (i+1) as i64) as f32;
		}

		return score;
	}

	fn get_possible_moves(board: &Vec<u8>, player_idx: u8) -> Vec<(u8, f32)> {
		let mut rval: Vec<(u8, f32)> = Vec::new();
		
		for i in 0..board.len() {
			if (board[i] == u8::MAX) {
				let mut tmp_board = board.clone();

				tmp_board[i] = player_idx;

				let mut score = Self::get_position_score(i);

				score += Self::get_heuristic(&tmp_board) as f32;

				rval.push((i as u8, score));
			}
		}

		return rval;
	}

	fn minimax(&self, depth: usize, board: &Vec<u8>, mut alpha: f32, mut beta: f32, is_maximizing: bool) -> (f32, Vec<u8>)
	{
		let mut moves = Vec::with_capacity(depth);
		moves.push(u8::MAX);

		if (depth == 0) {
			return (Self::get_heuristic(board), moves);
		}

		// no moves fallback
		let mut squares_checked = 0;

		if (is_maximizing)
		{
			let mut val = -INFINITY;

			for i in 0..board.len() {
				if (board[i] != u8::MAX) {
					continue;
				}

				if (depth > 3) {
					println!("PGR @D {}: {}", depth, i);
				}

				let mut new_board = board.clone();

				new_board[i] = is_maximizing as u8;

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);
				if (node_result.0 > val) {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i as u8;
					moves.append(&mut node_result.1);
				}

				if (val > beta) {
					break;
				}
				if (val > alpha) {
					alpha = val;
				}
			}
			return (val, moves);
		}
		else
		{
			let mut val = INFINITY;

			for i in 0..board.len() {
				if (board[i] != u8::MAX) {
					continue;
				}

				if (depth > 3) {
					println!("PGR @D {}: {}", depth, i);
				}

				let mut new_board = board.clone();

				new_board[i] = is_maximizing as u8;

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);

				if (node_result.0 < val) {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i as u8;
					moves.append(&mut node_result.1);
				}

				if (val < alpha) {
					break;
				}
				if (val < beta) {
					beta = val;
				}
			}
			return (val, moves);
		}
	}

	pub fn solve<'a>(&mut self) -> Result<u8, Error>
	{
		let res = self.minimax(6, &self.board, -INFINITY, INFINITY, true);

		println!("MLEN: {}", res.1.len());

		for m in &res.1 {
			println!("M: {}", m);
		}
		println!("SCORE: {}", res.0);

		return Ok(*res.1.first().unwrap());
	}
}