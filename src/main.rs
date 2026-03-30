use crate::map::Heuristic;

mod map;
mod search;


fn main() {
	let mut game = search::Engine::new();
	loop {
		game.run_search(6);
		let best_step = game.get_best_move();
		game.do_step(best_step.0, best_step.1);
		print!("{:?}", game);
	}
}
