pub const BOARD_DIM: usize = 19;
pub const BOARD_WIDTH: usize = BOARD_DIM;
pub const BOARD_SIZE: usize = BOARD_DIM * BOARD_DIM;

pub const DIRECTIONS: [[[i32; 2]; 2]; 4] = [
	[[-1, 0], [1, 0]], //x
	[[0, -1], [0, 1]], //y
	[[-1, -1], [1, 1]], //tlbr
	[[-1, 1], [1, -1]], //trbl
];

pub const CAPTURE_SCORES: [f32; 6] = [
	0.0,
	8.0,
	16.0,
	32.0,
	64.0,
	f32::INFINITY
];

pub const B0_SCORES: [f32; 6] = [
	1.0,
	2.0,
	4.0,
	16.0,
	64.0,
	f32::INFINITY
];

pub const B1_SCORES: [f32; 6] = [
	1.0,
	2.0,
	0.0, // can capture
	8.0,
	16.0,
	f32::INFINITY
];

pub const B2_SCORES: [f32; 6] = [
	0.0,
	0.0,
	0.0,
	0.0,
	0.0,
	f32::INFINITY
];