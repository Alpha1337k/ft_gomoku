use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::{SocketAddr, TcpStream};
use thiserror::Error;
use websocket::OwnedMessage;
use websocket::{
	server::upgrade::WsUpgrade,
	sync::{server::upgrade::Buffer, Reader, Writer},
};

use crate::{
	api::{
		calculate::handle_calculate, evaluate::handle_evaluate, hotseat_move::handle_hotseat_move,
		invalid_moves::handle_get_invalid_moves, utils::send_ws_message,
	},
	board::Board,
	piece::Piece,
	position::Position,
};

mod calculate;
mod evaluate;
mod hotseat_move;
mod invalid_moves;
mod utils;

#[derive(Deserialize)]
pub struct CalculateRequest {
	pub board: serde_json::Map<String, Value>,
	pub depth: usize,
	pub in_move: Option<Position>,
	pub player: Piece,
	pub captures: [usize; 2],
	pub is_hint: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct MoveFlat {
	pub score: f32,
	pub position: Position,
	pub order_idx: usize,
	pub cutoff_at: usize,
}

#[derive(Serialize)]
pub struct BoardUpdateResponse<'a> {
	board: &'a Board,
	captures: [usize; 2],
}

#[derive(Serialize, Deserialize)]
pub struct WSMessage {
	subject: String,
	request_id: Option<String>,
	data: serde_json::Value,
}

#[derive(Error, Clone, Debug)]
enum ServerError {
	#[error("Invalid protocol provided")]
	InvalidProtocol,

	#[error("Unknown Command provided: {value}")]
	UnknownCommand { value: String },
}

pub struct GomokuClient {
	receiver: Reader<TcpStream>,
	sender: Writer<TcpStream>,
	ip: SocketAddr,
}

impl GomokuClient {
	pub fn new(request: WsUpgrade<TcpStream, Option<Buffer>>) -> Result<GomokuClient> {
		if !request.protocols().contains(&"rust-websocket".to_string()) {
			request.reject().unwrap();
			return Err(ServerError::InvalidProtocol.into());
		}

		let client = request
			.use_protocol("rust-websocket")
			.accept()
			.map_err(|e| anyhow::anyhow!("{:?}", e))?;

		let ip = client.peer_addr()?;
		let (receiver, sender) = client.split()?;

		println!("Client {} connected", &ip);

		Ok(GomokuClient {
			receiver,
			sender,
			ip,
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
						"calculate" => {
							handle_calculate(&mut self.sender, request_id.clone(), message.data)
						}
						"inv_moves" => handle_get_invalid_moves(
							&mut self.sender,
							request_id.clone(),
							message.data,
						),
						"hotseat_move" => {
							handle_hotseat_move(&mut self.sender, request_id.clone(), message.data)
						}
						"evaluate" => {
							handle_evaluate(&mut self.sender, request_id.clone(), message.data)
						}
						_ => Err(ServerError::UnknownCommand {
							value: message.subject,
						}
						.into()),
					};

					if result.is_err() {
						let error = result.unwrap_err();

						println!("ft_gomoku: error: {}", error);

						send_ws_message(
							&mut self.sender,
							Some(request_id),
							"error",
							error.to_string(),
						)
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
