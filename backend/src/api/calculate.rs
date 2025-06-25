use std::net::TcpStream;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use websocket::sync::Writer;

use crate::{api::{get_moves, resolve_infinity, resolve_mate_depth, send_ws_message, BoardUpdateResponse, CalculateRequest, GomokuClient, MoveFlat}, heuristic::Heuristic, minimax::GomokuSolver, piece::Piece, position::Position};
use crate::piece::PieceWrap;

#[derive(Serialize, Deserialize)]
struct CalculationResponse
{
	moves: Vec<MoveFlat>,
	depth_hits: Vec<usize>,
	current_score: f32,
	score: f32,
	mate_in: Option<i32>
}


pub fn handle_calculate(sender: &mut Writer<TcpStream>, request_id: String, data: Value) -> Result<()> {
	let request: CalculateRequest = serde_json::from_value(data)?;

	let mut solver = GomokuSolver::from_request(&request)?;

	send_ws_message(sender, None, "boardUpdate", BoardUpdateResponse {
		board: &solver.board,
		captures: solver.captures
	})?;

	let result = solver.solve()?;

	let mut new_board = solver.board.clone();
	
	let capture_count = new_board.set_move(
		result.position,
		request.player.get_opposite(), None);

	let captures = [
		if request.player.get_opposite() == Piece::Max {solver.captures[0] + capture_count} else {solver.captures[0]}, 
		if request.player.get_opposite() == Piece::Min {solver.captures[1] + capture_count} else {solver.captures[1]}
	];

	let current_score = resolve_infinity(Heuristic::new(&new_board, &captures).get_heuristic());

	send_ws_message(sender, None, "boardUpdate", BoardUpdateResponse {
		board: &new_board,
		captures
	})?;

	let moves = get_moves(&result);
	let mate_in = resolve_mate_depth(&result.score, &moves);

	send_ws_message(sender, Some(request_id), "calculate", CalculationResponse {
		score: resolve_infinity(result.score),
		current_score: current_score,
		depth_hits: solver.depth_entries,
		moves,
		mate_in,
	})?;


	return Result::Ok(());
}