use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, f32::{INFINITY}, io::Error, net::{TcpStream}};

use serde_json::json;
use websocket::{sync::Writer};

use crate::{board::{Board, Piece, PieceWrap, Position}, heuristic::Heuristic, WSMessage};

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

	fn minimax(&mut self, depth: usize, board: &Board, mut alpha: f32, mut beta: f32, player: Piece) -> (f32, Vec<Position>)
	{
		let mut moves = Vec::<Position>::with_capacity(depth);
		let mut heuristic = Heuristic::from_board(board);

		let heuristical_score = heuristic.get_heuristic();

		println!("H: {}", heuristical_score);

		if depth == 0 || heuristical_score.is_infinite() {
			self.depth_zero_hits += 1;
			return (heuristical_score, moves);
		}

		let possible_moves = heuristic.get_moves(player);

		println!("MOVES: {}", possible_moves.len());

		return (0.0, moves);

		moves.push(Position::new(0, 0));

		let _squares_checked = 0;

		if player.is_max()
		{
			let mut val = -INFINITY;

			for i in possible_moves {

				if depth >= 3 {
					println!("PGR @D {}: {} D0: {}", depth, i.0, self.depth_zero_hits);
				}

				let mut new_board = board.clone()
					.set_move(i.0, Piece::Max, Some(i.1.1));

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, Piece::Min);

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

				let mut new_board = board.clone()
					.set_move(i.0, Piece::Max, Some(i.1.1));

				let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, Piece::Max);

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

		let res = self.minimax(4, &self.board.clone(), -INFINITY, INFINITY, ((self.turn_idx % 2) as u64).try_into().unwrap());

		for m in &res.1 {
			println!("M: {}", m);
		}
		println!("SCORE: {}", res.0);

		return Ok(res);
	}
}
