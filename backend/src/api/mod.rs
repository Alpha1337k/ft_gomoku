use std::{error::Error, net::{SocketAddr, TcpStream}, thread};
use serde::{Deserialize, Serialize};
use websocket::OwnedMessage;
use anyhow::{Result};
use thiserror::Error;
use websocket::{server::upgrade::WsUpgrade, sync::{server::upgrade::Buffer, Reader, Server, Writer}};
use serde_json::{Value};

use crate::{api::{calculate::handle_calculate, evaluate::handle_evaluate, hotseat_move::handle_hotseat_move, invalid_moves::handle_get_invalid_moves}, board::Board, game_move::Move, piece::Piece, position::Position};

mod calculate;
mod invalid_moves;
mod hotseat_move;
mod evaluate;


#[derive(Deserialize)]
pub struct CalculateRequest {
	pub board: serde_json::Map<String, Value>,
	pub depth: usize,
	pub in_move: Option<Position>,
	pub player: Piece,
	pub captures: [usize; 2],
	pub is_hint: Option<bool>
}

#[derive(Serialize, Deserialize)]
pub struct MoveFlat {
	pub score: f32,
	pub position: Position,
	pub order_idx: usize,
	pub cutoff_at: usize,
}

pub fn get_moves(res: &Move) -> Vec<MoveFlat> {
	let mut iter = res;

	let mut moves = Vec::<MoveFlat>::new();

	loop {
		if iter.child.is_none() {
			break;
		}

		println!("M: {}", iter.position);
		moves.push(MoveFlat {
			order_idx: iter.order_idx,
			position: iter.position,
			score: iter.score,
			cutoff_at: iter.cutoff_at
		});

		if let Some(child) = &iter.child.as_ref() {
			iter = child;
		} else {
			break;
		}
	}

	return moves;
}

// json not supporting infinity. Using magic numbers
pub fn resolve_infinity(val: f32) -> f32 {
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

pub fn send_ws_message<T: Serialize>(sender: &mut Writer<TcpStream>, request_id: Option<String>, subject: &str, data: T) -> Result<()> {
	let data_value = serde_json::to_value(data).expect("Failed to serialize data");

	let message = WSMessage {
		request_id,
		subject: subject.to_string(),
		data: data_value
	};

	let message = serde_json::to_string(&message)?;

	sender.send_message(&OwnedMessage::Text(message))?;

	Ok(())
}

pub fn resolve_mate_depth(score: &f32, moves: &Vec<MoveFlat>) -> Option<i32> {
	if score.is_finite() {
		return None;
	}

	let len = (moves.len() as i32 - 1).max(0);

	if score.is_sign_positive() {
		return Some(len);
	}
	return Some(-len);
}

#[derive(Serialize)]
pub struct BoardUpdateResponse<'a>
{
	board: &'a Board,
	captures: [usize; 2],
}

#[derive(Serialize, Deserialize)]
pub struct WSMessage
{
	subject: String,
	request_id: Option<String>,
	data: serde_json::Value
}

#[derive(Error, Clone, Copy, Debug)]
enum ServerError {
	#[error("Invalid protocol provided")]
	InvalidProtocol,

	#[error("Unknown Command provided: {value}")]
	UnknownCommand { value: String }
}

struct GomokuClient {
	receiver: Reader<TcpStream>,
	sender: Writer<TcpStream>,
	ip: SocketAddr
}

impl GomokuClient {
	pub fn send_message<T: Serialize>(&mut self, request_id: Option<String>, subject: &str, data: T) -> Result<()> {
		let data_value = serde_json::to_value(data).expect("Failed to serialize data");

		let message = WSMessage {
			request_id,
			subject: subject.to_string(),
			data: data_value
		};

		let message = serde_json::to_string(&message)?;

		self.sender.send_message(&OwnedMessage::Text(message))?;

		Ok(())
	}

	pub fn new(request: WsUpgrade<TcpStream, Option<Buffer>>) -> Result<GomokuClient> {
		if !request.protocols().contains(&"rust-websocket".to_string()) {
			request.reject().unwrap();
			return Err(ServerError::InvalidProtocol.into());
		}

		let client = request.use_protocol("rust-websocket").accept().map_err(|e| anyhow::anyhow!("{:?}", e))?;

		let ip = client.peer_addr()?;
		let (receiver, sender) = client.split()?;

		Ok(GomokuClient {
			receiver,
			sender,
			ip
		})
	}

	pub fn handle(&mut self) {
		loop {
			let message = match self.receiver.recv_message() {
				Ok(message) => message,
				Err(_) => break,
			};

			match message {
				OwnedMessage::Close(_) => {
					let message = OwnedMessage::Close(None);
					self.sender.send_message(&message).unwrap();
					println!("Client {} disconnected", self.ip);
					return;
				}
				OwnedMessage::Ping(ping) => {
					let message = OwnedMessage::Pong(ping);
					self.sender.send_message(&message).unwrap();
				}
				OwnedMessage::Text(text) => {
					let message: WSMessage = serde_json::from_str(&text).unwrap();
					let request_id = message.request_id.unwrap();

					let result = match message.subject.as_str() {
						"calculate" => handle_calculate(&mut self.sender, request_id.clone(), message.data),
						"inv_moves" => handle_get_invalid_moves(&mut self.sender, request_id.clone(), message.data),
						"hotseat_move" => handle_hotseat_move(&mut self.sender, request_id.clone(), message.data),
						"evaluate" => handle_evaluate(&mut self.sender, request_id.clone(), message.data),
						_ => Err(ServerError::UnknownCommand { value: message.subject }.into())
					};

					if result.is_err() {
						let error = result.unwrap_err();

						println!("ft_gomoku: error: {}", error);

						self.send_message(Some(request_id), "error", error.to_string())
							.unwrap_or_else(|_| {
								println!("ft_gomoku: error: failed to send error message to client");
							});
					}
				}
				_ => self.sender.send_message(&message).unwrap(),
			}
		}
	}
}
struct GomokuServer
{
	port: u16
}

impl GomokuServer {
	pub fn start(&self) -> Result<(), &str> {
		let address = format!("localhost:{}", self.port);

		let server = Server::bind(address).expect(&format!("Failed to bind server to port {}", self.port));

		for request in server.filter_map(Result::ok) {
			thread::spawn(|| {
				match GomokuClient::new(request) {
					Err(e) => println!("Failed to start new client: {}", e),
					Ok(mut c) => c.handle(),
				}
			});
		}

		Ok(())
	}
}