use std::{f32::{INFINITY}, io::Error, net::{TcpStream}};

use serde_json::json;
use websocket::{sync::Writer};

use crate::{board::{Board, Piece, PieceWrap, Position}, heuristic::Heuristic, WSMessage};

pub struct GomokuSolver<'a>
{
	pub board: Board,
	turn_idx: u8,
	depth: usize,
	sender: Option<&'a mut Writer<TcpStream>>,
	depth_zero_hits: usize
}

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: &'a mut Writer<TcpStream>) -> Option<GomokuSolver<'a>> {
		let board_raw = msg.data.as_object()?.get("board")?.as_object()?;

		let solver = GomokuSolver{
			board: Board::from_map(board_raw),
			turn_idx: msg.data.as_object()?.get("currentTurn").unwrap_or(&json!(0)).as_i64()? as u8,
			sender: Some(sender),
			depth_zero_hits: 0,
			depth: msg.data.as_object()?.get("depth").unwrap_or(&json!(0)).as_u64()? as usize,
		};

		return Some(solver);
	}

	fn minimax(&mut self, depth: usize, board: &Board, mut alpha: f32, mut beta: f32, player: Piece) -> (f32, Vec<Position>)
	{
		let mut moves = Vec::<Position>::with_capacity(depth);
		let mut heuristic = Heuristic::from_board(board);

		let heuristical_score = heuristic.get_heuristic();

		if depth == 0 || heuristical_score.is_infinite() {
			self.depth_zero_hits += 1;
			return (heuristical_score, moves);
		}

		let possible_moves = heuristic.get_moves(player);

		moves.push(Position::new(0, 0));

		let mut val;
		let opp_player = if player.is_max() {Piece::Min} else {Piece::Max};
				
		if player == Piece::Max {
			val = -INFINITY;
		} else {	
			val = INFINITY;
		}

		for i in possible_moves {
			let mut new_board = board.clone();

			println!("M: {}", i.0);

			new_board.set_move(i.0, player, Some(i.1.1));

			let mut node_result = self.minimax(depth - 1, &new_board, alpha, beta, opp_player);

			if depth == self.depth {
				println!("RES D: {}: pos: {} V:{}", self.depth, i.0, node_result.0);
			}

			if player.is_max() {
				if node_result.0 > val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i.0;
					moves.append(&mut node_result.1);
				}

				alpha = alpha.max(val);

				if val > beta {
					break;
				}
			} else {
				if node_result.0 < val {
					val = node_result.0;
					moves.truncate(1);
					moves[0] = i.0;
					moves.append(&mut node_result.1);
				}

				beta = beta.min(val);

				if val < alpha {
					break;
				}
			}

		}
		return (val, moves);
	}

	pub fn solve<'a>(&mut self) -> Result<(f32, Vec<Position>), Error>
	{
		println!("Starting minimax..");

		let res = self.minimax(self.depth, &self.board.clone(), -INFINITY, INFINITY, ((self.turn_idx % 2) as u64).try_into().unwrap());

		for m in &res.1 {
			println!("M: {}", m);
		}
		println!("SCORE: {}", res.0);

		return Ok(res);
	}
}
