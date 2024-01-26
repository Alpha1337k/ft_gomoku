use std::{collections::HashMap, io::Error, net::SocketAddr, thread::sleep, time::{Duration, SystemTime}};

use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use crate::{PeerMap, WSMessage};


pub struct GomokuSolver<'a>
{
	board: HashMap<u16, u8>,
	turn_idx: u8,
	sender: Option<SocketAddr>,
	socket_map: Option<&'a PeerMap>,
}

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: SocketAddr, socket_map: &'a PeerMap) -> Result<GomokuSolver<'a>, Error> {
		let board_raw = msg.data.as_object().unwrap().get("board").unwrap().as_object().unwrap();

		let mut solver = GomokuSolver{
			board: HashMap::new(),
			turn_idx: 0,
			sender: Some(sender),
			socket_map: Some(&socket_map)
		};
		
		for (key, value) in board_raw {
			let key_int = key.parse::<u16>().unwrap();
			solver.board.insert(key_int,value.as_i64().unwrap() as u8);
		}

		return Ok(solver);
	}

	pub fn solve(self) -> Result<u8, Error>
	{
		let mut depth = 0;
		let start_time = SystemTime::now();
		let last_update = SystemTime::now();

		while true {
			if (self.sender.is_some()
				&& SystemTime::now().duration_since(last_update).unwrap() >= Duration::from_secs(1)) {
				let message = serde_json::to_string(&WSMessage{
						subject: "CalculationUpdate".to_string(),
						requestId: None,
						data: serde_json::Value::Null
					}
				).unwrap();
				
				self.socket_map.unwrap().lock().unwrap().get(&self.sender.unwrap()).unwrap().unbounded_send(
					Message::Text(message)
				).unwrap();
			}

			println!("ROTATION");
			sleep(Duration::from_millis(200));


			if (SystemTime::now().duration_since(start_time).unwrap() >= Duration::from_secs(9)) {
				break;
			}
		}

		println!("Done!");

		return Ok(4);
	}
}