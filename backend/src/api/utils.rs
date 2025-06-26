use std::net::TcpStream;

use anyhow::Result;
use serde::Serialize;
use websocket::{sync::Writer, OwnedMessage};

use crate::{
	api::{MoveFlat, WSMessage},
	game_move::Move,
};

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
			cutoff_at: iter.cutoff_at,
		});

		if let Some(child) = &iter.child.as_ref() {
			iter = child;
		} else {
			break;
		}
	}

	moves
}

// json not supporting infinity. Using magic numbers
pub fn resolve_infinity(val: f32) -> f32 {
	if val.is_infinite() {
		if val.is_sign_negative() {
			return -1234.00;
		} else {
			return 1234.00;
		}
	}
	val
}

pub fn send_ws_message<T: Serialize>(
	sender: &mut Writer<TcpStream>,
	request_id: Option<String>,
	subject: &str,
	data: T,
) -> Result<()> {
	let data_value = serde_json::to_value(data).expect("Failed to serialize data");

	let message = WSMessage {
		request_id,
		subject: subject.to_string(),
		data: data_value,
	};

	let message = serde_json::to_string(&message)?;

	sender.send_message(&OwnedMessage::Text(message))?;

	Ok(())
}

pub fn resolve_mate_depth(score: &f32, moves: &[MoveFlat]) -> Option<i32> {
	if score.is_finite() {
		return None;
	}

	let len = (moves.len() as i32 - 1).max(0);

	if score.is_sign_positive() {
		Some(len)
	} else {
		Some(-len)
	}
}
