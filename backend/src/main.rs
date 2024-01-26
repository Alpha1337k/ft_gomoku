use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use serde::{Deserialize, Serialize};

use crate::algorithm::GomokuSolver;
mod algorithm;

#[derive(Serialize, Deserialize)]
pub struct WSMessage
{
	subject: String,
	requestId: Option<String>,
	data: serde_json::Value
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

			let mut client = request.use_protocol("rust-websocket").accept().unwrap();

			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

			let message = OwnedMessage::Text("{Hello}".to_string());
			client.send_message(&message).unwrap();

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

						if (message.subject == "calculate") {
							let mut solver = GomokuSolver::from_ws_msg(&message, &mut sender).unwrap();

							let result = solver.solve().unwrap();

							sender.send_message(&OwnedMessage::Text(
								serde_json::to_string(&WSMessage{
									requestId: message.requestId,
									subject: "calculate".to_string(),
									data: serde_json::Value::Number(result.into())
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