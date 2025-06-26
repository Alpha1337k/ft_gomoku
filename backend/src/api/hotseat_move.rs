use std::net::TcpStream;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use websocket::sync::Writer;

use crate::{
	api::{utils::resolve_infinity, utils::send_ws_message},
	board::Board,
	heuristic::Heuristic,
	piece::Piece,
	position::Position,
};

#[derive(Deserialize)]
struct HotseatRequest {
	board: serde_json::Map<String, Value>,
	player: Piece,
	in_move: Position,
	captures: [usize; 2],
}

#[derive(Serialize)]
struct HotseatResponse {
	board: Board,
	captures: [usize; 2],
	score: f32,
}

pub fn handle_hotseat_move(
	sender: &mut Writer<TcpStream>,
	request_id: String,
	data: Value,
) -> Result<()> {
	let request: HotseatRequest = serde_json::from_value(data)?;

	let mut board = Board::from_map(&request.board)?;

	let mut captures = request.captures;
	let capture_count = board.set_move(request.in_move, request.player, None);

	captures = [
		if request.player == Piece::Max {
			captures[0] + capture_count
		} else {
			captures[0]
		},
		if request.player == Piece::Min {
			captures[1] + capture_count
		} else {
			captures[1]
		},
	];

	let mut heuristic = Heuristic::new(&board, &[0, 0]);

	let score = heuristic.get_heuristic();

	send_ws_message(
		sender,
		Some(request_id),
		"hotseat_move",
		HotseatResponse {
			board,
			captures,
			score: resolve_infinity(score),
		},
	)?;

	Result::Ok(())
}
