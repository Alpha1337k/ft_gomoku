use rustc_hash::{FxHashMap, FxHashSet};
use crate::{block_position::BlockPosition, constants::DIRECTIONS, line::Line, piece::Piece, position::Position};


#[derive(Clone, Debug)]
pub struct LineStore {
	pub lines_idx: usize,
	pub lines: FxHashMap<usize, Line>,
	pub line_pos: FxHashMap<Position, [usize; 4]>,
}

impl Default for LineStore {
	fn default() -> Self {
		Self {
			lines_idx: 1,
			lines: FxHashMap::default(),
			line_pos: FxHashMap::default(),
		}
	}
}

impl LineStore {
	pub fn get_id(&self, id: &usize) -> Option<&Line> {
		self.lines.get(id)
	}

	pub fn get(&self, pos: &Position, direction_idx: usize) -> Option<&Line> {
		let lines = self.line_pos.get(pos);

		lines?;
		
		let line = self.lines.get(&lines?[direction_idx]);

		line
	}

	pub fn get_mut(&mut self, pos: &Position, direction_idx: usize) -> Option<&mut Line> {
		let lines = self.line_pos.get(pos);

		lines?;
		
		let line = self.lines.get_mut(&lines?[direction_idx]);

		line
	}

	fn populate_line_pos(&mut self, start: &Position, end: &Position, direction: [i32; 2], direction_idx: usize, reference_idx: usize) -> FxHashSet<usize>
	{
		let mut pos = *start;
		let mut overwritten_lines = FxHashSet::default();
		
		overwritten_lines.reserve(4);

		loop {	
			let p = if let std::collections::hash_map::Entry::Vacant(e) = self.entry(pos) {
				e.insert([0;4]);


				self.get_mut_pos(&pos).unwrap()
			} else {
				self.get_mut_pos(&pos).unwrap()
			};

			if p[direction_idx] != 0 {
				overwritten_lines.insert(p[direction_idx]);
			}
			
			p[direction_idx] = reference_idx;

			if pos == *end || pos.relocate(direction[0], direction[1]).is_err() {
				break;
			}

		}
		overwritten_lines
	}

	pub fn insert(&mut self, player: Piece, block_pos: BlockPosition, start: Position, end: Position, direction_idx: usize, length: usize) -> FxHashSet<usize> {
		self.lines_idx += 1;

		self.lines.insert(self.lines_idx, 
			Line::new(
				self.lines_idx,
				player,
				block_pos,
				start,
				end,
				direction_idx as u8,
				length
			)
		);

		let line = self.lines.get(&self.lines_idx).unwrap();

		self.populate_line_pos(
			&start, 
			&end, 
			DIRECTIONS[direction_idx][1], 
			direction_idx, 
			line.id
		)
	}

	pub fn values(&self) -> impl Iterator<Item = &Line> {
		self.lines.values()
	}

	pub fn remove(&mut self, idx: &usize) -> Option<Line> {
		self.lines.remove(&idx)
	}

	pub fn entry(&mut self, pos: Position) -> std::collections::hash_map::Entry<Position, [usize; 4]> {
		self.line_pos.entry(pos)
	}

	pub fn get_pos(&self, pos: &Position) -> Option<&[usize; 4]> {
		self.line_pos.get(pos)
	}

	fn get_mut_pos(&mut self, pos: &Position) -> Option<&mut [usize; 4]> {
		self.line_pos.get_mut(pos)
	}
}
