use crate::map::Heuristic;

mod map;
mod search;

const MAX_DEPTH: u64 = 8;

fn main() {
	let mut map = map::GameMap::new(0);
	let mut game = search::Engine::new(map);
	print!("{:?}", game);
	while game.won_by() == 0 {
		game.run_search(MAX_DEPTH);
		let (x, y) = game.get_best_move();
		game.place(x, y);
		print!("{:?}", game);
	}
}
