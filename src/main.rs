mod game_map;

use game_map::{GameMap, Tile};

fn main() {
	let mut game: GameMap = GameMap::new();
	game.set_tile(0, 0, Tile::X);
	game.set_tile(5, 5, Tile::X);
	println!("{}", game);
	game.set_tile(-5, -5, Tile::X);
	game.set_tile(5, -5, Tile::O);
	game.set_tile(-5, 5, Tile::O);
	println!("{}", game);
}
