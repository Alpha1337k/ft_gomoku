use iai_callgrind::{library_benchmark, library_benchmark_group, main};

extern crate backend;

use backend::position::Position;
use backend::board::Board;
use backend::minimax::GomokuSolver;
use backend::piece::Piece;

#[library_benchmark]
fn bench_calc() {

	let mut board = Board::new();

	board.set_move(Position::new(10, 10), Piece::Max, None);
	board.set_move(Position::new(11, 10), Piece::Min, None);
	board.set_move(Position::new(10, 12), Piece::Max, None);
	board.set_move(Position::new(14, 5), Piece::Min, None);
	board.set_move(Position::new(14, 14), Piece::Max, None);
	board.set_move(Position::new(7, 10), Piece::Min, None);


	let mut solver = GomokuSolver::from(GomokuSolver {
		board: board,
		is_hint: None,
		depth: 5,
		captures: [0,0],
		depth_entries: vec![0; 7],
		player: Piece::Max
	});

	let _ = solver.solve();
}

library_benchmark_group!(
    name = calc_bench;
    benchmarks = bench_calc
);

main!(library_benchmark_groups = calc_bench);