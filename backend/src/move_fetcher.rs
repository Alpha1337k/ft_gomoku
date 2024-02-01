// use std::collections::HashMap;


// pub struct Move {
// 	pos: usize,
// 	captures: u8,
// 	evaluation: f32
// }

// pub struct MoveList {
// 	board: Vec<usize>
// }

// impl MoveList {
// 	pub fn new(board: &Vec<usize>) {

// 	}

// 	fn check_capture(x: i32, y: i32, pos: usize, board: &Vec<usize>, opp_val: usize) -> bool
// 	{
// 		let mut old_idx = pos as i32;
// 		for i in 1..4 {
// 			let idx = (pos as i32) + (y * 19) * i + x * i;

// 			if idx >= 19 * 19 || 
// 				idx < 0 || 
// 				(old_idx % 19 == 18 && idx % 19 == 0) ||
// 				(old_idx % 19 == 0 && idx % 19 == 18) {
// 				return false;
// 			}

// 			if (i < 3 && board[idx as usize] != opp_val) {
// 				return false;
// 			}
// 			else if (i == 3 && board[idx as usize] == opp_val || board[idx as usize] == usize::MAX) {
// 				return false;
// 			}

// 			old_idx = idx;
// 		}

// 		return true;
// 	}

// 	fn evaluate_possible_move(board: &mut Vec<usize>, pos: usize, is_maximizing: bool) -> Option<(f32, u8)> {
// 		let coords = [
// 			[[-1, 0], [1, 0]], //x
// 			[[0, 1], [0, -1]], //y
// 			[[-1, -1], [1, 1]], //tlbr
// 			[[-1, 1], [1, -1]], //trbl
// 		];

// 		let mut total_score = 0.0;
// 		let mut capture_directions = 0u8;

// 		if (is_maximizing) {
// 			board[pos] = MAXIMIZING;
// 		} else {
// 			board[pos] = MINIMIZING;
// 		}

// 		println!("{}", board[pos]);

// 		for (i, direction) in coords.iter().enumerate() {
// 			let score_1 = Self::get_score(direction[0][0], direction[0][1], pos, board, None);
// 			let score_2 = Self::get_score(direction[1][0], direction[1][1], pos, board, None);
// 			let capture_1 = Self::check_capture(direction[0][0], direction[0][1], pos, board, if is_maximizing {MINIMIZING} else {MAXIMIZING});
// 			let capture_2 = Self::check_capture(direction[1][0], direction[1][1], pos, board, if is_maximizing {MINIMIZING} else {MAXIMIZING});


// 			let mut length = (score_1.0 + score_2.0 + 1) as usize;

// 			if (capture_1) {
// 				println!("CAPTURE1");
// 				capture_directions |= (1u8 << i);
// 				length *= 3;
// 			}
// 			if (capture_2) {
// 				println!("CAPTURE2");
// 				capture_directions |= (1u8 << i + 1);
// 				length *= 3;
// 			}

// 			println!("LENGTH: {} {}+{}+1", get_human_pos_name(pos as u8), score_1.0, score_2.0);

// 			if (length == 3 && score_1.1 == false && score_2.1 == false) {
// 				board[pos] = usize::MAX;
// 				return None;
// 			}

// 			total_score += (length.pow(2)) as f32;
// 		}

// 		board[pos] = usize::MAX;
// 		return Some((total_score, capture_directions));
// 	}

// 	fn get_possible_moves(board: &Vec<usize>, is_maximizing: bool) -> Vec<Move>
// 	{
// 		let mut position_set = HashMap::<usize, Option<(Move)>>::with_capacity(64);
// 		let mut board_clone = board.clone();

// 		for i in 0..board.len() {
// 			if board[i] != usize::MAX {
// 				for y in -1i32..2 {
// 					for x in -1i32..2 {
// 						let pos = i as i32 + (19 * y) + x;
// 						if (y == 0 && x == 0) ||
// 							pos < 0 ||
// 							pos >= 19 * 19 ||
// 							(x > 0 && i % 19 > (pos as usize) % 19) ||
// 							(x < 0 && i % 19 < (pos as usize) % 19)
// 						{
// 							continue;
// 						}
// 						if board[pos as usize] == usize::MAX && position_set.contains_key(&(pos as usize)) == false {
// 							let evaluation = Self::evaluate_possible_move(&mut board_clone, pos as usize, is_maximizing);
// 							if (evaluation.is_none()) {
// 								position_set.insert(pos as usize, None);
// 								continue;
// 							}
// 							position_set.insert(pos as usize, Some((
// 								evaluation.unwrap().0 + Self::get_position_score(pos as usize),
// 								evaluation.unwrap().1
// 								))
// 							);
// 						}
// 					}
// 				}
// 			}
// 		}

// 		let mut position_arr: Vec<(usize, (f32, u8))> = position_set.into_iter().filter(|x| x.1.is_some()).map(|f| (f.0, f.1.unwrap())).collect();

// 		position_arr.sort_by(|a, b| b.1.0.total_cmp(&a.1.0));
// 		position_arr.iter().for_each(|f| println!("{}: {} ({})", get_human_pos_name(f.0 as u8), f.1.0, f.1.1));


// 		return position_arr.iter().map(|v| (v.0, v.1.1)).collect();
// 	}

// 	pub fn fetch() -> Vec<Move> {

// 	}
// }