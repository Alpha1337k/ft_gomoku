use std::{net::TcpStream, thread};
use backend::minimax::Move;
use backend::piece::PieceWrap;
use backend::position::Position;
use backend::CalculateRequest;
use serde_json::{Value};
use websocket::sync::{Server, Writer};
use websocket::OwnedMessage;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use backend::{board::Board, heuristic::Heuristic, minimax::GomokuSolver, piece::Piece};

#[derive(Serialize, Deserialize)]
pub struct WSMessage
{
	subject: String,
	request_id: Option<String>,
	data: serde_json::Value
}

#[derive(Deserialize)]
pub struct EvalRequest {
	board: serde_json::Map<String, Value>,
	player: Piece
}

#[derive(Serialize, Deserialize)]
struct CalculationResponse
{
	moves: Vec<MoveFlat>,
	depth_hits: Vec<usize>,
	current_score: f32,
	score: f32,
	mate_in: Option<i32>
}

#[derive(Deserialize)]
struct PosMoveRequest {
	board: serde_json::Map<String, Value>,
	player: Piece
}

#[derive(Deserialize)]
struct HotseatRequest {
	board: serde_json::Map<String, Value>,
	player: Piece,
	in_move: Position,
	captures: [usize; 2]
}


#[derive(Serialize)]
struct HotseatResponse {
	board: Board,
	captures: [usize; 2],
	score: f32
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
	board_score: f32,
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

fn resolve_mate_depth(score: &f32, moves: &Vec<MoveFlat>) -> Option<i32> {
	if score.is_finite() {
		return None;
	}

	let len = (moves.len() as i32 - 1).max(0);

	if score.is_sign_positive() {
		return Some(len);
	}
	return Some(-len);
}

fn get_moves(res: &Move) -> Vec<MoveFlat> {
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

		if iter.child.is_some() {
			iter = iter.child.as_ref().unwrap().as_ref();
		} else {
			break;
		}
	}

	return moves;
}

fn handle_pos_moves(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) -> Result<()> {
	let request: PosMoveRequest = serde_json::from_value(data)?;

	let board = Board::from_map(&request.board);

	let mut heuristic = Heuristic::from_board(&board, &[0, 0]);

	heuristic.get_heuristic();

	let moves: Vec<Position> = heuristic.get_invalid_moves(request.player);

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			request_id,
			subject: "inv_moves".to_string(),
			data: serde_json::to_value(&moves)?
		})?
	))?;

	Result::Ok(())
}

fn handle_hotseat_move(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) -> Result<()> {
	let request: HotseatRequest = serde_json::from_value(data)?;

	let mut board = Board::from_map(&request.board);

	let mut captures = request.captures.clone();
	let capture_count = board.set_move(request.in_move, request.player, None);

	captures = [
		if request.player == Piece::Max {captures[0] + capture_count} else {captures[0]}, 
		if request.player == Piece::Min {captures[1] + capture_count} else {captures[1]}
	];

	let mut heuristic = Heuristic::from_board(&board, &[0, 0]);

	let score = heuristic.get_heuristic();

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage {
			request_id,
			subject: "hotseat_move".to_string(),
			data: serde_json::to_value(&HotseatResponse {
				board: board,
				captures: captures,
				score: resolve_infinity(score)
			})?
		})?
	))?;

	Result::Ok(())
}

fn handle_calculate(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) -> Result<()> {
	let request: CalculateRequest = serde_json::from_value(data)?;

	let mut solver = GomokuSolver::from_request(&request);

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			request_id: None,
			subject: "boardUpdate".to_string(),
			data: serde_json::to_value(&BoardUpdateResponse {
				board: &solver.board,
				captures: solver.captures
			})?
		})?
	))?;

	let result = solver.solve()?;

	let mut new_board = solver.board.clone();
	
	let capture_count = new_board.set_move(
		result.position,
		request.player.get_opposite(), None);

	let captures = [
			if request.player.get_opposite() == Piece::Max {solver.captures[0] + capture_count} else {solver.captures[0]}, 
			if request.player.get_opposite() == Piece::Min {solver.captures[1] + capture_count} else {solver.captures[1]}
	];

	let current_score = resolve_infinity(Heuristic::from_board(&new_board, &captures).get_heuristic());

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			request_id: None,
			subject: "boardUpdate".to_string(),
			data: serde_json::to_value(&BoardUpdateResponse {
				board: &new_board,
				captures: captures
			})?
		})?
	))?;

	let moves = get_moves(&result);
	let mate_in = resolve_mate_depth(&result.score, &moves);

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			request_id,
			subject: "calculate".to_string(),
			data: serde_json::to_value(CalculationResponse{
				score: resolve_infinity(result.score),
				current_score: current_score,
				depth_hits: solver.depth_entries,
				moves,
				mate_in,
			})?
		})?
	))?;

	return Result::Ok(());
}

fn handle_evaluate(sender: &mut Writer<TcpStream>, request_id: Option<String>, data: Value) -> Result<()> {
	let request: EvalRequest = serde_json::from_value(data)?;
	let board = Board::from_map(&request.board);

	let mut heuristic = Heuristic::from_board(&board, &[0, 0]);

	let board_score = heuristic.get_heuristic();
	let mut moves = heuristic.get_moves(request.player);

	for i in 0..moves.len() {
		moves[i].1.score = resolve_infinity(moves[i].1.score);
	}

	println!("Evaluating done");

	sender.send_message(&OwnedMessage::Text(
		serde_json::to_string(&WSMessage{
			request_id,
			subject: "evaluate".to_string(),
			data: serde_json::to_value(EvaluationResponse{
				board_score: resolve_infinity(board_score),
				moves: moves.iter().map(|f| (f.0, (f.1.score, f.1.capture_map))).collect()
			})?
		})?
	))?;

	Result::Ok(())
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

						match message.subject.as_str() {
							"calculate" => handle_calculate(&mut sender, message.request_id, message.data),
							"inv_moves" => handle_pos_moves(&mut sender, message.request_id, message.data),
							"hotseat_move" => handle_hotseat_move(&mut sender, message.request_id, message.data),
							"evaluate" => handle_evaluate(&mut sender, message.request_id, message.data),
							_ => panic!("ft_gomoku: error: command not found: {}", message.subject)
						}.unwrap()
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}
