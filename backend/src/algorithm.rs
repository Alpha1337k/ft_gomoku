use std::{collections::{HashMap, HashSet}, io::Error, net::{SocketAddr, TcpStream}, thread::sleep, time::{Duration, SystemTime}};

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
			visited_places.insert(idx as usize);

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

	fn get_heuristic(board: &Vec<u8>) -> i64 {
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

		let mut score: i64 = 0;

		for i in 0..5 {
			score += ((p0_scores[i] - p1_scores[i]) as i64 * (i+1) as i64);
		}

		return score;
	}

	fn get_possible_moves(board: &Vec<u8>, player_idx: u8) -> Vec<(u8, i64)> {
		let mut rval: Vec<(u8, i64)> = Vec::new();
		
		for i in 0..board.len() {
			if (board[i] == u8::MAX) {
				let mut tmp_board = board.clone();

				tmp_board[i] = player_idx;

				rval.push((i as u8, Self::get_heuristic(&tmp_board)));
			}
		}

		return rval;
	}

	pub fn solve<'a>(&mut self) -> Result<u8, Error>
	{
		let mut depth = 0;
		let start_time = SystemTime::now();
		let last_update = SystemTime::now();

		loop {
			if (self.sender.is_some()
				&& SystemTime::now().duration_since(last_update).unwrap() >= Duration::from_secs(1)) {
				let message = serde_json::to_string(&WSMessage{
						subject: "CalculationUpdate".to_string(),
						requestId: None,
						data: serde_json::Value::Null
					}
				).unwrap();
								
				self.sender.as_deref_mut().unwrap().send_message(&OwnedMessage::Text(message)).unwrap();
			}

			let mut moves = Self::get_possible_moves(&self.board, self.turn_idx);

			println!("Moves: {}", self.board.len());

			moves.sort_by(|a, b| a.1.cmp(&b.1));

			println!("Best: {}, {}", moves.first().unwrap().0, moves.first().unwrap().1);
			println!("Worst: {}, {}", moves.last().unwrap().0, moves.last().unwrap().1);

			// for movey in moves {
			// 	println!("{}, {}", movey.0, movey.1);
			// }

			depth += 1;

			if (
				depth >= 10
				||
				SystemTime::now().duration_since(start_time).unwrap() >= Duration::from_secs(9)) {
				break;
			}
		}

		println!("Done!");

		return Ok(4);
	}
}