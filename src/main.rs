mod map;
mod game_params;
mod heurs;

fn main() {
	let mut game = map::GameMap::new();
	game.set_tile(0, 0, map::Tile::X);
	game.set_tile(5, 5, map::Tile::X);
	game.set_tile(-5, -5, map::Tile::X);
	game.set_tile(5, -5, map::Tile::O);
	game.set_tile(-5, 5, map::Tile::O);
	game.set_tile(3, 3, map::Tile::X);
	println!("{:?}", game);
	let mut checker = heurs::GameThreats::();
	let _ = game.run_line_check(&mut checker);
	println!("{:?}", checker);
}
