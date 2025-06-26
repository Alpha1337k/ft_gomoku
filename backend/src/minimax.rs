use crate::api::CalculateRequest;
use crate::{
	board::{Board, BoardError},
	game_move::{print_moveset, Move},
	heuristic::{EvaluationScore, Heuristic},
	piece::{Piece, PieceWrap},
	position::Position,
};
use thiserror::Error;

pub struct GameState {
	pub board: Board,
	pub captures: [usize; 2],
	pub player: Piece,
}

struct MiniMaxResult {
	found_better_move: bool,
	prune_trigger: bool,
	prune_threshold: f32,
}

pub struct GomokuSolver {
	pub board: Board,
	pub captures: [usize; 2],
	pub depth: usize,
	pub player: Piece,
	pub depth_entries: Vec<usize>,
	pub is_hint: Option<bool>,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum MinMaxError {
	#[error("Game is already over")]
	GameOver,
}

impl GomokuSolver {
	pub fn from_request(msg: &CalculateRequest) -> Result<GomokuSolver, BoardError> {
		let mut solver = GomokuSolver {
			board: Board::from_map(&msg.board)?,
			captures: msg.captures,
			depth_entries: vec![0; msg.depth + 1],
			depth: msg.depth,
			player: msg.player,
			is_hint: msg.is_hint,
		};

		if let Some(in_move) = msg.in_move {
			let capture_count = solver.board.set_move(in_move, msg.player, None);

			solver.captures = [
				if msg.player == Piece::Max {
					solver.captures[0] + capture_count
				} else {
					solver.captures[0]
				},
				if msg.player == Piece::Min {
					solver.captures[1] + capture_count
				} else {
					solver.captures[1]
				},
			];
		}

		Ok(solver)
	}

	fn compare_move(
		node_result: &Move,
		best_move: &Move,
		mut prune_threshold: f32,
		end_score: f32,
		cmp: &dyn Fn(f32, f32) -> bool,
		prune_op: &dyn Fn(f32, f32) -> f32,
	) -> MiniMaxResult {
		let mut prune_trigger = false;
		let mut found_better_move = false;

		if cmp(node_result.score, best_move.score)
			|| (best_move.score == end_score && node_result.depth_hit > best_move.depth_hit)
		{
			found_better_move = true;
		}

		prune_threshold = prune_op(prune_threshold, node_result.score);

		if cmp(best_move.score, prune_threshold) {
			prune_trigger = true;
		}

		MiniMaxResult {
			found_better_move,
			prune_trigger,
			prune_threshold,
		}
	}

	fn minimax(
		&mut self,
		mut depth: usize,
		old_heuristic: &Heuristic,
		state: &GameState,
		mut alpha: f32,
		mut beta: f32,
	) -> Move {
		let mut heuristic = old_heuristic.from_new_state(state);
		let mut found_move = false;

		self.depth_entries[self.depth - depth] += 1;
		let heuristical_score = heuristic.get_heuristic();

		if depth == 0 || heuristical_score.is_infinite() {
			if depth != 0
				&& state.captures[state.player as usize] == 4
				&& ((state.player.is_min() && heuristical_score.is_sign_positive())
					|| (state.player.is_max() && heuristical_score.is_sign_negative()))
			{
				depth = 1;
			} else {
				return Move {
					score: heuristical_score,
					depth_hit: depth,
					depth_score: heuristical_score,
					captures: heuristic.captures,
					..Move::default()
				};
			}
		}

		let mut possible_moves = heuristic.get_moves(state.player);

		if possible_moves.is_empty() && state.board[&Position::new(10, 10)].is_empty() {
			possible_moves.push((
				Position::new(10, 10),
				EvaluationScore {
					capture_count: 0,
					capture_map: 0,
					score: 0.0,
				},
			));
		} else if possible_moves.is_empty() {
			panic!("No possible starter move found")
		}

		let mut best_move = Move {
			score: if state.player.is_max() {
				-f32::INFINITY
			} else {
				f32::INFINITY
			},
			depth_hit: depth,
			depth_score: heuristical_score,
			captures: heuristic.captures,
			cutoff_at: possible_moves.len(),
			..Move::default()
		};

		for (i, pos_move) in possible_moves.iter().enumerate() {
			if !heuristic.validate_move(pos_move.0, state.player) {
				continue;
			}

			let mut new_board = state.board.clone();

			let capture_count =
				new_board.set_move(pos_move.0, state.player, Some(pos_move.1.capture_map));

			let game_state = GameState {
				board: new_board,
				captures: [
					if state.player == Piece::Max {
						heuristic.captures[0] + capture_count
					} else {
						heuristic.captures[0]
					},
					if state.player == Piece::Min {
						heuristic.captures[1] + capture_count
					} else {
						heuristic.captures[1]
					},
				],
				player: state.player.get_opposite(),
			};

			let node_result = self.minimax(depth - 1, &heuristic, &game_state, alpha, beta);

			if depth == self.depth {
				println!(
					"RES D: {}: pos: {} PRED: {} V:{}",
					node_result.depth_hit, pos_move.0, pos_move.1.score, node_result.score
				);
				print_moveset(&pos_move.0, i, heuristical_score, &node_result);
			}

			let end_score = if state.player.is_max() {
				f32::INFINITY
			} else {
				-f32::INFINITY
			};

			let result = match state.player {
				Piece::Max => {
					let res = GomokuSolver::compare_move(
						&node_result,
						&best_move,
						beta,
						end_score,
						&|a, b| a > b,
						&|a, b| a.max(b),
					);

					alpha = res.prune_threshold;

					res
				}
				Piece::Min => {
					let res = GomokuSolver::compare_move(
						&node_result,
						&best_move,
						alpha,
						end_score,
						&|a, b| a < b,
						&|a, b| a.min(b),
					);

					beta = res.prune_threshold;

					res
				}
				_ => panic!("Invalid player type in minimax"),
			};

			if result.found_better_move || !found_move {
				found_move = true;
				best_move.update(i, node_result, pos_move);
			}

			if result.prune_trigger
				|| (depth != self.depth && (result.prune_threshold == end_score))
			{
				best_move.cutoff_at = i;
				break;
			}
		}
		best_move
	}

	pub fn solve(&mut self) -> Result<Move, MinMaxError> {
		println!(
			"Starting minimax.. as player {}\n",
			if self.is_hint.is_some_and(|x| x) {
				self.player
			} else {
				self.player.get_opposite()
			}
		);

		let game_state = GameState {
			board: self.board.clone(),
			captures: self.captures,
			player: if self.is_hint.is_some_and(|x| x) {
				self.player
			} else {
				self.player.get_opposite()
			},
		};

		let mut heuristic = Heuristic::new(&game_state.board, &game_state.captures);

		let base_score = heuristic.get_heuristic();

		if base_score.is_infinite() {
			return Err(MinMaxError::GameOver);
		}

		let res = self.minimax(
			self.depth,
			&heuristic,
			&game_state,
			-f32::INFINITY,
			f32::INFINITY,
		);

		println!("----");
		print_moveset(&res.position, res.order_idx, base_score, &res);

		println!(
			"SCORE: {} - depth: {:?} = {}",
			res.score,
			&self.depth_entries,
			&self.depth_entries.iter().sum::<usize>()
		);

		Ok(res)
	}
}
