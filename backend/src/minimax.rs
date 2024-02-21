use std::{f32::{INFINITY}, io::Error};

use serde::{Deserialize, Serialize};
use serde_json::json;


use crate::{board::Board, heuristic::Heuristic, move_calculator::{self, MoveCalculator}, piece::{Piece, PieceWrap}, position::Position, CalculateRequest, WSMessage};

pub struct Move {
	pub score: f32,
	pub position: Position,
	pub order_idx: usize,
	pub cutoff_at: usize,
	pub child: Option<Box<Move>>,
}

pub struct GomokuSolver
{
	pub board: Board,
	turn_idx: u8,
	depth: usize,
	pub depth_entries: Vec<usize>
}

impl GomokuSolver {
	pub fn from_request(msg: &CalculateRequest) -> GomokuSolver {

		let mut solver = GomokuSolver{
			board: Board::from_map(&msg.board),
			turn_idx: msg.turn_idx,
			depth_entries: vec![0; msg.depth + 1],
			depth: msg.depth,
		};

		if (msg.in_move.is_some()) {
			solver.board.set_move(msg.in_move.unwrap(), msg.player, None);
		}

		return solver;
	}

	fn minimax(&mut self, depth: usize, board: &Board, mut alpha: f32, mut beta: f32, player: Piece) -> Move
	{
		let mut move_store = Move {
			child: None,
			cutoff_at: 0,
			score: if player == Piece::Max {-INFINITY} else {INFINITY},
			order_idx: 0,
			position: Position::new(0, 0)
		};

		let mut heuristic = Heuristic::from_board(board);


		self.depth_entries[depth] += 1;
		let heuristical_score = heuristic.get_heuristic();

		if depth == 0 || heuristical_score.is_infinite() {
			return Move {
				child: None,
				cutoff_at: 0,
				score: heuristical_score,
				order_idx: 0,
				position: Position::new(0, 0)
			};
		}

		let m_calc = MoveCalculator::new(&board);
		let possible_moves = heuristic.get_moves(player, &m_calc.moves);

		for (i, pos_move) in possible_moves.iter().enumerate() {
			let mut new_board = board.clone();

			// println!("MC: {}", pos_move.0);

			new_board.set_move(pos_move.0, player, Some(pos_move.1.1));

			let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, player.get_opposite());

			if depth == self.depth {
				println!("RES D: {}: pos: {} PRED: {} V:{}", self.depth, pos_move.0, pos_move.1.0, node_result.score);
			}

			if player.is_max() {
				if node_result.score > move_store.score {
					move_store.score = node_result.score;
					move_store.position = pos_move.0;
					move_store.order_idx = i;
					move_store.child = Some(Box::new(node_result));
				}

				alpha = alpha.max(move_store.score);

				if move_store.score > beta {
					move_store.cutoff_at = i;
					break;
				}
			} else {
				if node_result.score < move_store.score {
					move_store.score = node_result.score;
					move_store.position = pos_move.0;
					move_store.order_idx = i;
					move_store.child = Some(Box::new(node_result));
				}

				beta = beta.min(move_store.score);

				if move_store.score < alpha {
					move_store.cutoff_at = i;
					break;
				}
			}

		}
		return move_store;
	}

	pub fn solve<'a>(&mut self) -> Result<Move, Error>
	{
		println!("Starting minimax..");

		let res = self.minimax(self.depth, &self.board.clone(), -INFINITY, INFINITY, Piece::Min);

		let mut iter = &res;

		loop {
			println!("M: {}", iter.position);
			if (iter.child.is_some()) {
				iter = iter.child.as_ref().unwrap().as_ref();
			} else {
				break;
			}
		}

		println!("SCORE: {}", res.score);

		return Ok(res);
	}
}
