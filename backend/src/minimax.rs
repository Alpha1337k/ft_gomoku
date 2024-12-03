use std::{f32::INFINITY, io::Error, usize};
use crate::{board::Board, heuristic::{EvaluationScore, Heuristic}, piece::{Piece, PieceWrap}, position::Position, CalculateRequest};


fn print_moveset(base_pos: &Position, base_order_idx: usize, base_score: f32, m: &Move) {
	let mut iter = m;

	print!("Start score: {}, moves: ( {} ({}) [{}, {}] = {} )", base_score, base_pos, base_order_idx, iter.captures[0], iter.captures[1] , iter.depth_score);

	loop {
		if iter.child.is_none() {
			break;
		}
		print!(" -> ");
		print!("( {} ({}) [{}, {}]", iter.position, iter.order_idx, iter.captures[0], iter.captures[1]);
		
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
	pub depth_hit: usize,
	pub position: Position,
	pub capture_map: u8,
	pub captures: [usize; 2],
	pub order_idx: usize,
	pub cutoff_at: usize,
	pub child: Option<Box<Move>>,
}

impl Move {
	pub fn make_half_empty(score: f32, depth_hit: usize, depth_score: f32, captures: &[usize; 2]) -> Move {
		Move {
			child: None,
			cutoff_at: 0,
			score,
			depth_score,
			depth_hit,
			order_idx: 0,
			capture_map: 0,
			captures: captures.clone(),
			position: Position::new(0, 0)			
		}
	}

	pub fn update(&mut self, order: usize, child_move: Move, pos_move: &(Position, EvaluationScore)) {
		self.score = child_move.score;
		self.position = pos_move.0;
		self.order_idx = order;
		self.capture_map = pos_move.1.capture_map;
		self.depth_hit = child_move.depth_hit;
		self.child = Some(Box::new(child_move));
	}
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
	pub depth: usize,
	pub player: Piece,
	pub depth_entries: Vec<usize>,
	pub is_hint: Option<bool>
}

impl GomokuSolver {
	pub fn from_request(msg: &CalculateRequest) -> GomokuSolver {

		let mut solver = GomokuSolver{
			board: Board::from_map(&msg.board),
			captures: msg.captures,
			depth_entries: vec![0; msg.depth + 1],
			depth: msg.depth,
			player: msg.player,
			is_hint: msg.is_hint
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

	fn minimax(&mut self, mut depth: usize, old_heuristic: &Heuristic, state: &GameState, mut alpha: f32, mut beta: f32) -> Move
	{
		let mut heuristic = old_heuristic.from_new_state(&state);
		let mut found_move = false;

		self.depth_entries[self.depth - depth] += 1;
		let heuristical_score = heuristic.get_heuristic();

		let mut move_store = Move::make_half_empty(
			if state.player.is_max() {-INFINITY} else {INFINITY},
			 depth, heuristical_score, &heuristic.captures);

		if depth == 0 || heuristical_score.is_infinite() {
			if depth != 0 && state.captures[state.player as usize] == 4 && (
				(state.player.is_min() && heuristical_score.is_sign_positive()) ||
				(state.player.is_max() && heuristical_score.is_sign_negative())
			)
			{
				depth = 1;
			}
			else {
				return Move::make_half_empty(heuristical_score, depth, heuristical_score, &heuristic.captures);
			}
		}

		let mut possible_moves = heuristic.get_moves(state.player);

		if possible_moves.len() == 0 && state.board[&Position::new(10, 10)].is_empty() {
			possible_moves.push((Position::new(10, 10), EvaluationScore {
				capture_count: 0,
				capture_map: 0,
				score: 0.0
			}));
		} else if possible_moves.len() == 0 {
			panic!("No possible starter move found")
		}

		move_store.cutoff_at = possible_moves.len();

		for (i, pos_move) in possible_moves.iter().enumerate() {
			let mut new_board = state.board.clone();

			if heuristic.validate_move(pos_move.0, state.player) == false {
				continue;
			}

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
				println!("RES D: {}: pos: {} PRED: {} V:{}", node_result.depth_hit, pos_move.0, pos_move.1.score, node_result.score);
				print_moveset(&pos_move.0, i, heuristical_score, &node_result);
			}

			if state.player.is_max() {
				if node_result.score > move_store.score || 
					(found_move == false) ||
					(move_store.score == INFINITY && node_result.depth_hit > move_store.depth_hit) {
					
					found_move = true;
					move_store.update(i, node_result, pos_move);
				}

				alpha = alpha.max(move_store.score);

				if move_store.score > beta || 
					(depth != self.depth && move_store.score == INFINITY) {
					move_store.cutoff_at = i;
					break;
				}
			} else {
				if node_result.score < move_store.score || 
					(found_move == false) ||
					(move_store.score == -INFINITY && node_result.depth_hit > move_store.depth_hit) {
					found_move = true;
					move_store.update(i, node_result, pos_move);
				}

				beta = beta.min(move_store.score);

				if move_store.score < alpha ||
					(depth != self.depth && move_store.score == -INFINITY)
				 {
					move_store.cutoff_at = i;
					break;
				}
			}
		}
		return move_store;
	}

	pub fn solve<'a>(&mut self) -> Result<Move, Error>
	{
		println!("Starting minimax.. as player {}\n", if self.is_hint.is_some_and(|x| x == true) { self.player } else {self.player.get_opposite() });

		let game_state = GameState {
			board: self.board.clone(),
			captures: self.captures,
			player: if self.is_hint.is_some_and(|x| x == true) { self.player } else {self.player.get_opposite() },
		};

		let mut heuristic = Heuristic::from_game_state(&game_state);

		let base_score = heuristic.get_heuristic();

		let res = self.minimax(self.depth, &heuristic, &game_state, -INFINITY, INFINITY);

		println!("----");
		print_moveset(&res.position, res.order_idx, base_score, &res);

		println!("SCORE: {} - depth: {:?} = {}", res.score, &self.depth_entries, &self.depth_entries.iter().sum::<usize>());

		return Ok(res);
	}
}
