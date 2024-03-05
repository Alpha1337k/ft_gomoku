use std::{f32::{INFINITY}, io::Error};
use crate::{board::Board, heuristic::{EvaluationScore, Heuristic}, piece::{Piece, PieceWrap}, position::Position, CalculateRequest};


fn print_moveset(base_pos: &Position, base_order_idx: usize, _base_captures: u8, base_score: f32, m: &Move) {
	let mut iter = m;

	print!("Start score: {}, moves: ( {} ({}) [{}] = {} )", base_score, base_pos, base_order_idx, iter.captures , iter.depth_score);

	loop {
		if iter.child.is_none() {
			break;
		}
		print!(" -> ");
		print!("( {} ({}) [{}]", iter.position, iter.order_idx, iter.captures);
		
		if iter.child.is_some() {
			iter = iter.child.as_ref().unwrap().as_ref();
			print!(" = {})", iter.depth_score);
		}
	}
	println!();
}


#[derive(Debug)]
pub struct Move {
	pub score: f32,
	pub depth_score: f32,
	pub position: Position,
	pub captures: u8,
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
	depth: usize,
	pub player: Piece,
	pub depth_entries: Vec<usize>
}

impl GomokuSolver {
	pub fn from_request(msg: &CalculateRequest) -> GomokuSolver {

		let mut solver = GomokuSolver{
			board: Board::from_map(&msg.board),
			captures: msg.captures,
			depth_entries: vec![0; msg.depth + 1],
			depth: msg.depth,
			player: msg.player,
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

	fn minimax(&mut self, depth: usize, old_heuristic: &Heuristic, state: &GameState, mut alpha: f32, mut beta: f32) -> Move
	{
		let mut heuristic = old_heuristic.from_new_state(&state);

		self.depth_entries[depth] += 1;
		let heuristical_score = heuristic.get_heuristic();

		let mut move_store = Move {
			child: None,
			cutoff_at: 1234,
			score: if state.player == Piece::Max {-INFINITY} else {INFINITY},
			depth_score: heuristical_score,
			order_idx: 0,
			captures: 0,
			position: Position::new(0, 0)
		};

		// if (depth == 0) {
			let mut heuristic_check = Heuristic::from_game_state(&state);
			let heurstic_check_score = heuristic_check.get_heuristic();

			if heuristical_score != heurstic_check_score {
				println!("MISMATCH HEURISTICAL VALUES: {} vs {}", heuristical_score, heurstic_check_score);
				println!("{}", state.board);
				println!("{}", old_heuristic.board);
				
				dbg!(heuristic.lines);
				dbg!(heuristic_check.lines);
				dbg!(&old_heuristic.lines);
				panic!();
			}
		// }


		if depth == 0 || heuristical_score.is_infinite() {
			return Move {
				child: None,
				cutoff_at: 0,
				score: heuristical_score,
				depth_score: heuristical_score,
				captures: 0,
				order_idx: 0,
				position: Position::new(0, 0)
			};
		}

		let mut possible_moves = heuristic.get_moves(state.player);

		if possible_moves.len() == 0 && state.board[&Position::new(10, 10)].is_empty() {
			possible_moves.push((Position::new(10, 10), EvaluationScore {
				capture_count: 0,
				capture_map: 0,
				score: 0.0
			}));
		} else if possible_moves.len() == 0 {
			panic!()
		}

		for (i, pos_move) in possible_moves.iter().enumerate() {
			let mut new_board = state.board.clone();

			if heuristic.validate_move(pos_move.0, state.player) == false {
				continue;
			}

			// println!("\nMC: {} {}", pos_move.0, pos_move.1.capture_map);

			let capture_count = new_board.set_move(pos_move.0, state.player, Some(pos_move.1.capture_map));

			let node_result = self.minimax(depth - 1, &heuristic, &GameState {
				board: new_board,
				captures: [
					if state.player == Piece::Max {heuristic.captures[0] + capture_count} else {heuristic.captures[0]}, 
					if state.player == Piece::Min {heuristic.captures[1] + capture_count} else {heuristic.captures[1]}
				],
				player: state.player.get_opposite(),
			}, alpha, beta);

			if depth == self.depth {
				println!("RES D: {}: pos: {} PRED: {} V:{}", depth, pos_move.0, pos_move.1.score, node_result.score);
				print_moveset(&pos_move.0, i, pos_move.1.capture_map, heuristical_score, &node_result);
			}

			if state.player.is_max() {
				if node_result.score >= move_store.score {
					move_store.score = node_result.score;
					move_store.position = pos_move.0;
					move_store.order_idx = i;
					move_store.captures = pos_move.1.capture_map;
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
					move_store.captures = pos_move.1.capture_map;
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
		println!("Starting minimax.. as player\n");

		let game_state = GameState {
			board: self.board.clone(),
			captures: self.captures,
			player: self.player.get_opposite(),
		};

		let mut heuristic = Heuristic::from_game_state(&game_state);

		let _ = heuristic.get_heuristic();

		let res = self.minimax(self.depth, &heuristic, &game_state, -INFINITY, INFINITY);

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
