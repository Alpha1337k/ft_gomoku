use std::{collections::HashMap, io::Error, net::{SocketAddr, TcpStream}, thread::sleep, time::{Duration, SystemTime}};

use websocket::{sync::Writer, OwnedMessage};

use crate::{WSMessage};


pub struct GomokuSolver<'a>
{
	board: HashMap<u16, u8>,
	turn_idx: u8,
	sender: Option<&'a mut Writer<TcpStream>>,
}

impl GomokuSolver<'_> {
	pub fn from_ws_msg<'a>(msg: &WSMessage, sender: &'a mut Writer<TcpStream>) -> Result<GomokuSolver<'a>, Error> {
		let board_raw = msg.data.as_object().unwrap().get("board").unwrap().as_object().unwrap();

		let mut solver = GomokuSolver{
			board: HashMap::new(),
			turn_idx: 0,
			sender: Some(sender),
		};
		
		for (key, value) in board_raw {
			let key_int = key.parse::<u16>().unwrap();
			solver.board.insert(key_int,value.as_i64().unwrap() as u8);
		}

		return Ok(solver);
	}

	pub fn solve<'a>(&mut self) -> Result<u8, Error>
	{
		let mut depth = 0;
		let start_time = SystemTime::now();
		let last_update = SystemTime::now();

		loop {
			if (self.sender.is_some()
				&& SystemTime::now().duration_since(last_update).unwrap() >= Duration::from_secs(1)) {
				let message = serde_json::to_string(&WSMessage{
						subject: "CalculationUpdate".to_string(),
						requestId: None,
						data: serde_json::Value::Null
					}
				).unwrap();
								
				self.sender.as_deref_mut().unwrap().send_message(&OwnedMessage::Text(message)).unwrap();
			}

			println!("ROTATION");
			sleep(Duration::from_millis(200));

			depth += 1;

			if (SystemTime::now().duration_since(start_time).unwrap() >= Duration::from_secs(9)) {
				break;
			}
		}

		println!("Done!");

		return Ok(4);
	}
}