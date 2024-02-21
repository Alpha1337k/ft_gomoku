use std::{net::TcpStream, thread};

use minimax::Move;
use piece::PieceWrap;
use position::Position;
use serde_json::{Value};
use websocket::sync::{Server, Writer};
use websocket::OwnedMessage;
use serde::{Deserialize, Serialize};

use crate::{board::Board, heuristic::Heuristic, minimax::GomokuSolver, move_calculator::{MoveCalculator}, piece::Piece};
mod minimax;
mod board;
mod position;
mod piece;
mod heuristic;
mod move_calculator;

#[derive(Serialize, Deserialize)]
pub struct WSMessage
{
	subject: String,
	requestId: Option<String>,
	data: serde_json::Value
}

#[derive(Deserialize)]
pub struct EvalRequest {
	board: serde_json::Map<String, Value>,
	player: Piece
}

#[derive(Deserialize)]
pub struct CalculateRequest {
	board: serde_json::Map<String, Value>,
	depth: usize,
	turn_idx: u8,
	in_move: Option<Position>,
	player: Piece,
	captures: [usize; 2],
}

#[derive(Serialize, Deserialize)]
struct CalculationResponse
{
	moves: Vec<MoveFlat>,
	depth_hits: Vec<usize>,
	score: f32,
}

#[derive(Serialize)]
struct BoardUpdateResponse<'a>
{
	board: &'a Board,
	captures: [usize; 2],
}

#[derive(Serialize, Deserialize)]
struct EvaluationResponse
{
	moves: Vec<(Position, (f32, u8))>,
	boardScore: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MoveFlat {
	pub score: f32,
	pub position: Position,
	pub order_idx: usize,
	pub cutoff_at: usize,
}

// json not supporting infinity. Using magic numbers
fn resolve_infinity(val: f32) -> f32 {
	if val.is_infinite() {
		if val.is_sign_negative() {
			return -1234.00;
		}
		else {
			return 1234.00;
		}
	}
	return val;
}

fn get_moves(res: &Move) -> Vec<MoveFlat> {
	let mut iter = res;

	let mut moves = Vec::<MoveFlat>::new();

	loop {
		println!("M: {}", iter.position);
		moves.push(MoveFlat {
			order_idx: iter.order_idx,
			position: iter.position,
			score: iter.score,
			cutoff_at: iter.cutoff_at
		});

		if iter.child.is_some() {
			iter = iter.child.as_ref().unwrap().as_ref();
		} else {
			break;
		}
	}

	return moves;
}

fn handle_calculate(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) {
	let request: CalculateRequest = serde_json::from_value(data).unwrap();

	let mut solver = GomokuSolver::from_request(&request);

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			requestId: None,
			subject: "boardUpdate".to_string(),
			data: serde_json::to_value(&BoardUpdateResponse {
				board: &solver.board,
				captures: solver.captures
			}).unwrap()
		}).unwrap()
	)).unwrap();

	let result = solver.solve().unwrap();

	let mut new_board = solver.board.clone();
	
	let capture_count = new_board.set_move(
		result.position,
		request.player.get_opposite(), None);

	let captures = [
			if request.player.get_opposite() == Piece::Max {solver.captures[0] + capture_count} else {solver.captures[0]}, 
			if request.player.get_opposite() == Piece::Min {solver.captures[1] + capture_count} else {solver.captures[1]}
	];

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			requestId: None,
			subject: "boardUpdate".to_string(),
			data: serde_json::to_value(&BoardUpdateResponse {
				board: &new_board,
				captures: captures
			}).unwrap()
		}).unwrap()
	)).unwrap();

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			requestId: request_id,
			subject: "calculate".to_string(),
			data: serde_json::to_value(CalculationResponse{
				score: resolve_infinity(result.score),
				depth_hits: solver.depth_entries,
				moves: get_moves(&result),
			}).unwrap()
		}).unwrap()
	)).unwrap();
}

fn main() {
	let server = Server::bind("localhost:8000").unwrap();

	for request in server.filter_map(Result::ok) {
		// Spawn a new thread for each connection.
		thread::spawn(|| {
			if !request.protocols().contains(&"rust-websocket".to_string()) {
				request.reject().unwrap();
				return;
			}

			let client = request.use_protocol("rust-websocket").accept().unwrap();

			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

			let (mut receiver, mut sender) = client.split().unwrap();

			let mut old_board = Board::new();
			let _moves = MoveCalculator::new(&old_board);

			for message in receiver.incoming_messages() {
				let message = message.unwrap();

				match message {
					OwnedMessage::Close(_) => {
						let message = OwnedMessage::Close(None);
						sender.send_message(&message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					}
					OwnedMessage::Ping(ping) => {
						let message = OwnedMessage::Pong(ping);
						sender.send_message(&message).unwrap();
					}
					OwnedMessage::Text(text) => {
						let message: WSMessage = serde_json::from_str(&text).unwrap();

						if message.subject == "calculate" {
							handle_calculate(&mut sender, message.requestId, message.data);
						} else if message.subject == "evaluate" {
							let request: EvalRequest = serde_json::from_value(message.data).unwrap();
							let board = Board::from_map(&request.board);

							let mut heuristic = Heuristic::from_board(&board);

							let board_score = heuristic.get_heuristic();
							let mut moves = heuristic.get_moves(request.player);

							for i in 0..moves.len() {
								moves[i].1.0 = resolve_infinity(moves[i].1.0);
							}

							println!("Evaluating done");

							old_board = board;

							sender.send_message(&OwnedMessage::Text(
								serde_json::to_string(&WSMessage{
									requestId: message.requestId,
									subject: "evaluate".to_string(),
									data: serde_json::to_value(EvaluationResponse{
										boardScore: resolve_infinity(board_score),
										moves: moves
									}).unwrap()
								}).unwrap()
							)).unwrap();
						}
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}