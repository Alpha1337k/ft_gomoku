use std::{collections::{HashMap, HashSet}, f32::INFINITY, io::Error, net::{TcpStream}};

use serde_json::json;
use websocket::{sync::Writer};

use crate::{WSMessage};

pub struct GomokuSolver<'a>
{
	pub board: Vec<usize>,
	turn_idx: u8,
	sender: Option<&'a mut Writer<TcpStream>>,
}

const MAXIMIZING: usize = 0;
const MINIMIZING: usize = 1;

fn get_human_pos_name(pos: u8) -> String {
	let y_char = (pos / 19) + 65;

	return format!("{}{}", y_char as char, pos % 19);
}


static mut DEPTHZERO_HITS: usize = 0;

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: &'a mut Writer<TcpStream>) -> Result<GomokuSolver<'a>, Error> {
		let board_raw = msg.data.as_object().unwrap().get("board").unwrap().as_object().unwrap();

		let mut solver = GomokuSolver{
			board: vec![usize::MAX; 19 * 19],
			turn_idx: msg.data.as_object().unwrap().get("currentTurn").unwrap_or(&json!(0)).as_i64().unwrap() as u8,
			sender: Some(sender),
		};
		
		for (key, value) in board_raw {
			let key_int = key.parse::<usize>().unwrap();
			solver.board[key_int] = value.as_u64().unwrap() as usize;
		}

		return Ok(solver);
	}

	pub fn get_possible_moves(board: &Vec<usize>) -> Vec<usize>
	{
		let mut position_set = HashMap::<usize, f32>::with_capacity(64);

		for i in 0..board.len() {
			if board[i] != usize::MAX {
				for y in -1i32..2 {
					for x in -1i32..2 {
						let pos = i as i32 + (19 * y) + x;
						if (y == 0 && x == 0) ||
							pos < 0 ||
							pos >= 19 * 19 ||
							(x > 0 && i % 19 > (pos as usize) % 19) ||
							(x < 0 && i % 19 < (pos as usize) % 19)
						{
							continue;
						}
						if board[pos as usize] == usize::MAX && position_set.contains_key(&(pos as usize)) == false {
							position_set.insert(pos as usize, Self::get_position_score(pos as usize));
						}
					}
				}
			}
		}

		let mut position_arr: Vec<(usize, f32)> = position_set.into_iter().collect();

		position_arr.sort_by(|a, b| b.1.total_cmp(&a.1));

		return position_arr.iter().map(|v| v.0).collect();
	}

	fn get_score(x: i32, y: i32, start_idx: usize, board: &Vec<usize>, visited_places: &mut HashSet<usize>) -> (u8, bool)
	{
		let mut idx: i32 = start_idx as i32;
		let mut len = 0;
		loop {
			let old_idx = idx;
			idx += y * 19 + x;

			if idx >= 19 * 19 || 
				idx < 0 || 
				(old_idx % 19 == 18 && idx % 19 == 0) ||
				(old_idx % 19 == 0 && idx % 19 == 18) {
				return (len, true);
			}

			if board[idx as usize] != board[start_idx] {
				if board[idx as usize] == usize::MAX {
					return (len, false);
				}
				return (len, true);
			}

			visited_places.insert(idx as usize);

			len += 1;
		}
	}

	fn get_surround_score(visited_places: &mut HashSet<usize>, start_idx: usize, board: &Vec<usize>) -> f32 {
		let coords = [
			[[-1, 0], [1, 0]], //x
			[[0, 1], [0, -1]], //y
			[[-1, -1], [1, 1]], //tlbr
			[[-1, 1], [1, -1]], //trbl
		];

		let mut total_score = 0.0;

		for (_i, direction) in coords.iter().enumerate() {
			let score_1 = Self::get_score(direction[0][0], direction[0][1], start_idx, board, visited_places);
			let score_2 = Self::get_score(direction[1][0], direction[1][1], start_idx, board, visited_places);

			let length = (score_1.0 + score_2.0 + 1) as usize;
			let mut score: f32 = length as f32;


			if length >= 5 {
				return INFINITY;
			}

			score *= length as f32;

			if score_1.1 == false && score_2.1 == false {
				score *= 1.4;
			}

			else if score_1.1 == true && score_2.1 == true {
				score = 0.0;
			}

			total_score += score;
		}

		return total_score;
	}

	fn get_position_score(pos: usize) -> f32 {
		let y = 1f32 - ((9.5 - (pos as f32 / 19f32).floor()).abs() / 9.5f32);
		let x = 1f32 - ((9.5 - (pos % 19) as f32).abs() / 9.5f32);

		return (y + x) / 2f32;
	}

	pub fn get_position_scores(board: &Vec<usize>) -> f32 {
		let mut score = 0.0;
		
		for i in 0..board.len() {
			if board[i] == MAXIMIZING {
				score += Self::get_position_score(i);
			} else if board[i] == MINIMIZING {
				score -= Self::get_position_score(i);
			}
		}

		return score;
	}

	fn get_heuristic(board: &Vec<usize>) -> f32 {
		let mut maximizing_score = 0.0;
		let mut minimizing_score = 0.0;
		let mut visited_places = HashSet::<usize>::with_capacity(19 * 19);

		for i in 0..board.len() {
			if minimizing_score == INFINITY || maximizing_score == INFINITY {
				return maximizing_score - minimizing_score;
			}

			if board[i] == usize::MAX || visited_places.get(&i).is_some() {
				continue;
			}
			if board[i] == MAXIMIZING {
				maximizing_score = Self::get_surround_score(&mut visited_places, i, board);
			} else if board[i] == MINIMIZING {
				minimizing_score = Self::get_surround_score(&mut visited_places, i, board);
			}
		}

		let mut score: f32 = Self::get_position_scores(board);

		score += maximizing_score - minimizing_score;

		return score;
	}

	fn minimax(&self, depth: usize, board: &Vec<usize>, mut alpha: f32, mut beta: f32, is_maximizing: bool) -> (f32, Vec<usize>)
	{
		let mut moves = Vec::with_capacity(depth);
		let heuristic = Self::get_heuristic(board);

		if depth == 0 || heuristic.is_infinite() {
			unsafe { DEPTHZERO_HITS += 1 };
			return (heuristic, moves);
		}

		let possible_moves = Self::get_possible_moves(board);

		moves.push(usize::MAX);

		// println!("POS_MOVES: {}", possible_moves.len());

		// possible_moves.truncate(80);

		// no moves fallback
		let _squares_checked = 0;

		if is_maximizing
		{
			let mut val = -INFINITY;

			for i in possible_moves {
				if board[i] != usize::MAX {
					continue;
				}

				if depth >= 3 {
					println!("PGR @D {}: {} D0: {}", depth, i, unsafe {DEPTHZERO_HITS});
				}

				let mut new_board = board.clone();

				new_board[i] = MAXIMIZING;

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);
				if node_result.0 > val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i;
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
				if board[i] != usize::MAX {
					continue;
				}

				if depth >= 3 {
					println!("PGR @D {}: {} D0: {}", depth, i, unsafe {DEPTHZERO_HITS});
				}

				let mut new_board = board.clone();

				new_board[i] = MINIMIZING;

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, !is_maximizing);

				// println!("RES: pos: {} V:{}", get_human_pos_name(i as u8), node_result.0);

				if node_result.0 < val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i;
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

	pub fn solve<'a>(&mut self) -> Result<(f32, Vec<usize>), Error>
	{
		println!("Lets GO: {} {}", Self::get_position_score(0), Self::get_position_score(180));

		unsafe {
			DEPTHZERO_HITS = 0;
		}

		let res = self.minimax(4, &self.board, -INFINITY, INFINITY, self.turn_idx % 2 == 0);

		for m in &res.1 {
			println!("M: {}", m);
		}
		println!("SCORE: {}", res.0);

		return Ok(res);
	}
}
