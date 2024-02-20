use std::{net::TcpStream, thread};

use piece::PieceWrap;
use position::Position;
use serde_json::{json, Value};
use websocket::sync::{Server, Writer};
use websocket::OwnedMessage;
use serde::{Deserialize, Serialize};

use crate::{board::Board, heuristic::Heuristic, minimax::GomokuSolver, move_calculator::{Move, MoveCalculator}, piece::Piece};
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
pub struct MoveRequest {
	delta: Vec<Move>,
	board: serde_json::Map<String, Value>
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
	player: Piece
}

#[derive(Serialize, Deserialize)]
struct CalculationResponse
{
	moves: Vec<u64>,
	score: f32,
}

#[derive(Serialize, Deserialize)]
struct EvaluationResponse
{
	moves: Vec<(Position, (f32, u8))>,
	boardScore: f32,
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

fn handle_calculate(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) {
	let request: CalculateRequest = serde_json::from_value(data).unwrap();

	let mut solver = GomokuSolver::from_request(&request);

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			requestId: None,
			subject: "boardUpdate".to_string(),
			data: serde_json::to_value(&solver.board).unwrap()
		}).unwrap()
	)).unwrap();

	let result = solver.solve().unwrap();

	let mut new_board = solver.board.clone();
	
	new_board.set_move(
		*result.1.first().unwrap(),
		request.player.get_opposite(), None);

		sender.send_message(&OwnedMessage::Text(
			serde_json::to_string(&WSMessage{
				requestId: None,
				subject: "boardUpdate".to_string(),
				data: serde_json::to_value(&new_board).unwrap()
			}).unwrap()
		)).unwrap();

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			requestId: request_id,
			subject: "calculate".to_string(),
			data: serde_json::to_value(CalculationResponse{
				score: resolve_infinity(result.0),
				moves: result.1.iter().map(|x| x.to_u64()).collect(),
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
			let mut moves = MoveCalculator::new(&old_board);

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

							moves = MoveCalculator::new(&board);

							println!("Positions evaluated: {} PM_MX: {} PM_MN: {}", moves.positions_checked, moves.moves[0].len(),
							moves.moves[1].len()
						);

							let mut heuristic = Heuristic::from_board(&board);

							let board_score = heuristic.get_heuristic();
							let moves = heuristic.get_moves(request.player, &moves.moves);

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
						} else if message.subject == "moves" {
							let request: MoveRequest = serde_json::from_value(message.data).unwrap();
							let board = Board::from_map(&request.board);

							moves = MoveCalculator::from_calculator(&moves, &board, request.delta);

							sender.send_message(&OwnedMessage::Text(
								serde_json::to_string(&WSMessage{
									requestId: message.requestId,
									subject: "moves".to_string(),
									data: serde_json::to_value(&moves).unwrap()
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