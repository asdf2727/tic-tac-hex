mod map;
mod heurs;
pub mod search;

fn main() {
	let mut game = map::GameMap::new::<heurs::GameThreats>();
	game.set_tile(0, 0, map::Tile::X);
	game.set_tile(5, 5, map::Tile::X);
	game.set_tile(-5, -5, map::Tile::X);
	game.set_tile(5, -5, map::Tile::O);
	game.set_tile(-5, 5, map::Tile::O);
	game.set_tile(3, 3, map::Tile::X);
	game.set_tile(2, 2, map::Tile::X);
	game.set_tile(1, 1, map::Tile::O);
	println!("{:?}", game);
	println!("{:?}", game.get_heuristic());
}
