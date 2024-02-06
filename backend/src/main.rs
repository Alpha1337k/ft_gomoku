use std::{collections::HashMap, thread};
use board::Position;
use serde_json::json;
use websocket::sync::Server;
use websocket::OwnedMessage;
use serde::{Deserialize, Serialize};

use crate::{algorithm::GomokuSolver, board::Piece, heuristic::Heuristic};
mod algorithm;
mod move_fetcher;
mod board;
mod heuristic;

#[derive(Serialize, Deserialize)]
pub struct WSMessage
{
	subject: String,
	requestId: Option<String>,
	data: serde_json::Value
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
	evalPrio: Vec<u64>,
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
							let mut solver = GomokuSolver::from_ws_msg(&message, &mut sender).unwrap();

							let mut result = solver.solve().unwrap();

							sender.send_message(&OwnedMessage::Text(
								serde_json::to_string(&WSMessage{
									requestId: message.requestId,
									subject: "calculate".to_string(),
									data: serde_json::to_value(CalculationResponse{
										score: resolve_infinity(result.0),
										moves: result.1.iter().map(|x| x.to_u64()).collect(),
									}).unwrap()
								}).unwrap()
							)).unwrap();
						} else if message.subject == "evaluate" {
							let solver = GomokuSolver::from_ws_msg(&message, &mut sender).unwrap();
							let mut heuristic = Heuristic::from_board(&solver.board);

							let player_bool = message.data.get("is_maximizing").unwrap_or(&json!(true)).as_bool().unwrap();

							let board_score = heuristic.get_heuristic();
							let moves = heuristic.get_moves( 
								if player_bool {Piece::Max} else {Piece::Min}
								);

							println!("Evaluating done");

							sender.send_message(&OwnedMessage::Text(
								serde_json::to_string(&WSMessage{
									requestId: message.requestId,
									subject: "evaluate".to_string(),
									data: serde_json::to_value(EvaluationResponse{
										boardScore: resolve_infinity(board_score),
										evalPrio: moves.iter().map(|f| f.0.to_u64()).collect()
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