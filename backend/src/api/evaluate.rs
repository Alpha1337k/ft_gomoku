use std::net::TcpStream;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use websocket::sync::Writer;

use crate::{api::{resolve_infinity, send_ws_message, GomokuClient}, board::Board, heuristic::Heuristic, piece::Piece, position::Position};

#[derive(Deserialize)]
pub struct EvalRequest {
	board: serde_json::Map<String, Value>,
	player: Piece
}

#[derive(Serialize, Deserialize)]
struct EvaluationResponse
{
	moves: Vec<(Position, (f32, u8))>,
	board_score: f32,
}

pub fn handle_evaluate(sender: &mut Writer<TcpStream>, request_id: String, data: Value) -> Result<()> {
	let request: EvalRequest = serde_json::from_value(data)?;
	let board = Board::from_map(&request.board)?;

	let mut heuristic = Heuristic::new(&board, &[0, 0]);

	let board_score = heuristic.get_heuristic();
	let mut moves = heuristic.get_moves(request.player);

	for i in 0..moves.len() {
		moves[i].1.score = resolve_infinity(moves[i].1.score);
	}

	println!("Evaluating done");

	send_ws_message(sender, Some(request_id), "evaluate", EvaluationResponse {
		board_score: resolve_infinity(board_score),
		moves: moves.iter().map(|f| (f.0, (f.1.score, f.1.capture_map))).collect()		
	})?;

	Result::Ok(())
}