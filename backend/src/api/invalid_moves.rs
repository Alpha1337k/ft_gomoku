use std::net::TcpStream;

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use websocket::sync::Writer;

use crate::{api::{utils::send_ws_message}, board::Board, heuristic::Heuristic, piece::Piece, position::Position};

#[derive(Deserialize)]
struct InvMovesRequest {
	board: serde_json::Map<String, Value>,
	player: Piece
}

pub fn handle_get_invalid_moves(sender: &mut Writer<TcpStream>, request_id: String, data: Value) -> Result<()> {
	let request: InvMovesRequest = serde_json::from_value(data)?;

	let board = Board::from_map(&request.board)?;
	let heuristic = Heuristic::new(&board, &[0, 0]);
	let moves: Vec<Position> = heuristic.get_invalid_moves(request.player);

	send_ws_message(sender, Some(request_id), "inv_moves", moves)?;

	Result::Ok(())
}