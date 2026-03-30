use super::*;
use heurs::*;
use quad_root::*;
use std::fmt::{Debug, Formatter};

pub type Hash = u64;

pub struct GameMap {
	tree: QuadRoot,
	heuristic: GameMeasure,
	hash: Hash,
	move_list: Vec<(i64, i64)>,
}

impl GameMap {
	pub fn new() -> GameMap {
		let mut gm = GameMap {
			tree: QuadRoot::new(),
			heuristic: GameMeasure::new(),
			hash: 0,
			move_list: Vec::new(),
		};
		gm.update(0, 0, Tile::X);
		gm.heuristic.update_step(1);
		gm
	}

	pub fn get_heuristic(&self) -> &GameMeasure {
		&self.heuristic
	}

	pub fn get_hash(&self) -> Hash { self.hash }

	pub fn get_move_list(&self) -> &Vec<(i64, i64)> { &self.move_list }

	/// See https://github.com/lemire/testingRNG/blob/master/source/splitmix64.h
	const fn splitmix64(mut x: Hash) -> Hash {
		x = x.wrapping_add(0x9E3779B97F4A7C15);
		x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
		x ^ (x >> 31)
	}
	fn get_tile_hash(x: i64, y: i64, tile: Tile) -> Hash {
		let tile_hash: Hash = match tile {
			Tile::Empty => return 0,
			Tile::X => 0xC85A0D3A7E3B7508,
			Tile::O => 0x3658570FED0B5A86,
		};
		Self::splitmix64((x ^ (y << 32)) as Hash ^ tile_hash)
	}

	pub fn get_tile(&self, x: i64, y: i64) -> Tile { self.tree.get_tile(x, y) }

	fn update(&mut self, x: i64, y: i64, tile: Tile) {
		self.heuristic.update(&mut self.tree, x, y, -1);
		self.hash ^= Self::get_tile_hash(x, y, self.tree.set_tile(x, y, tile));
		self.hash ^= Self::get_tile_hash(x, y, tile);
		self.heuristic.update(&mut self.tree, x, y, 1);
	}

	pub fn place(&mut self, x: i64, y: i64, tile: Tile) {
		debug_assert!(!matches!(tile, Tile::Empty), "Don't place empty tiles! Use undo_move instead.");
		self.move_list.push((x, y));
		self.update(x, y, tile);
		self.heuristic.update_step(1);
	}
	pub fn undo(&mut self) {
		let (x, y) = self.move_list.pop().unwrap();
		self.update(x, y, Tile::Empty);
		self.heuristic.update_step(-1);
	}

	pub fn peek_hash(&self, x: i64, y: i64, tile: Tile) -> Hash {
		self.hash ^ Self::get_tile_hash(x, y, tile)
	}

	pub fn clean_tree(&mut self) {
		self.tree.clean();
	}
}

impl Debug for GameMap {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.tree.fmt(f)?;
		self.heuristic.fmt(f)
	}
}