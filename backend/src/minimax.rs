use std::{f32::{INFINITY}, io::Error};





use crate::{board::Board, heuristic::Heuristic, piece::{Piece, PieceWrap}, position::Position, CalculateRequest};

pub struct Move {
	pub score: f32,
	pub position: Position,
	pub order_idx: usize,
	pub cutoff_at: usize,
	pub child: Option<Box<Move>>,
}

pub struct GameState {
	pub board: Board,
	pub captures: [usize; 2],
	pub player: Piece,
}

pub struct GomokuSolver
{
	pub board: Board,
	pub captures: [usize; 2],
	turn_idx: u8,
	depth: usize,
	pub depth_entries: Vec<usize>
}

impl GomokuSolver {
	pub fn from_request(msg: &CalculateRequest) -> GomokuSolver {

		let mut solver = GomokuSolver{
			board: Board::from_map(&msg.board),
			turn_idx: msg.turn_idx,
			captures: msg.captures,
			depth_entries: vec![0; msg.depth + 1],
			depth: msg.depth,
		};

		if msg.in_move.is_some() {
			let capture_count = solver.board.set_move(msg.in_move.unwrap(), msg.player, None);
		
			solver.captures = [
				if msg.player == Piece::Max {solver.captures[0] + capture_count} else {solver.captures[0]}, 
				if msg.player == Piece::Min {solver.captures[1] + capture_count} else {solver.captures[1]}
			];
		}

		return solver;
	}

	fn minimax(&mut self, depth: usize, state: &GameState, mut alpha: f32, mut beta: f32) -> Move
	{
		let mut move_store = Move {
			child: None,
			cutoff_at: 0,
			score: if state.player == Piece::Max {-INFINITY} else {INFINITY},
			order_idx: 0,
			position: Position::new(0, 0)
		};

		let mut heuristic = Heuristic::from_game_state(&state);


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

		// let m_calc = MoveCalculator::new(&board);
		let possible_moves = heuristic.get_moves(state.player);

		for (i, pos_move) in possible_moves.iter().enumerate() {
			let mut new_board = state.board.clone();

			// println!("MC: {}", pos_move.0);

			let capture_count = new_board.set_move(pos_move.0, state.player, Some(pos_move.1.1));

			let node_result = self.minimax(depth - 1, &GameState {
				board: new_board,
				captures: [
					if state.player == Piece::Max {self.captures[0] + capture_count} else {self.captures[0]}, 
					if state.player == Piece::Min {self.captures[1] + capture_count} else {self.captures[1]}
				],
				player: state.player.get_opposite(),
			}, alpha, beta);

			if depth >= self.depth - 1 {
				println!("RES D: {}: pos: {} PRED: {} V:{}", depth, pos_move.0, pos_move.1.0, node_result.score);
			}

			if state.player.is_max() {
				if node_result.score >= move_store.score {
					move_store.score = node_result.score;
					move_store.position = pos_move.0;
					move_store.order_idx = i;
					move_store.child = Some(Box::new(node_result));
				}

				alpha = alpha.max(move_store.score);

				if move_store.score > beta || move_store.score == INFINITY {
					move_store.cutoff_at = i;
					break;
				}
			} else {
				if node_result.score <= move_store.score {
					move_store.score = node_result.score;
					move_store.position = pos_move.0;
					move_store.order_idx = i;
					move_store.child = Some(Box::new(node_result));
				}

				beta = beta.min(move_store.score);

				if move_store.score < alpha || move_store.score == -INFINITY {
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

		let res = self.minimax(self.depth, &GameState {
			board: self.board.clone(),
			captures: self.captures,
			player: Piece::Min,
		}, -INFINITY, INFINITY);

		let mut iter = &res;

		loop {
			println!("M: {}", iter.position);
			if iter.child.is_some() {
				iter = iter.child.as_ref().unwrap().as_ref();
			} else {
				break;
			}
		}

		println!("SCORE: {}", res.score);

		return Ok(res);
	}
}
