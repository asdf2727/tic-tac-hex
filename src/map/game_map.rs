use super::*;
use heurs::*;
use quad_root::*;
use std::fmt::{Debug, Formatter};

pub type Hash = u64;

pub struct GameMap {
	tree: QuadRoot,
	heuristic: GameMeasure,
	move_list: Vec<(i64, i64)>,
	hash: Hash,
	step: u64,
}

impl GameMap {
	pub fn new(start_step: u64) -> GameMap {
		let mut gm = GameMap {
			tree: QuadRoot::new(),
			heuristic: GameMeasure::new(),
			move_list: Vec::new(),
			hash: 0,
			step: start_step,
		};
		gm.place_init(0, 0, Tile::X);
		gm
	}

	pub fn get_heuristic(&self) -> &GameMeasure {
		&self.heuristic
	}

	pub fn get_move_list(&self) -> &Vec<(i64, i64)> { &self.move_list }

	pub fn get_depth(&self) -> u64 { self.step }
	pub fn get_player(&self) -> Tile {
		if self.step & 2 != 0 { Tile::X } else { Tile::O }
	}

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

	pub fn get_hash(&self) -> Hash { self.hash }

	pub fn peek_hash(&self, x: i64, y: i64) -> Hash {
		self.hash ^ Self::get_tile_hash(x, y, self.get_player())
	}

	pub fn get_tile(&self, x: i64, y: i64) -> Tile { self.tree.get_tile(x, y) }

	fn update(&mut self, x: i64, y: i64, tile: Tile) {
		self.heuristic.update(&mut self.tree, x, y, -1);
		self.hash ^= Self::get_tile_hash(x, y, self.tree.set_tile(x, y, tile));
		self.hash ^= Self::get_tile_hash(x, y, tile);
		self.heuristic.update(&mut self.tree, x, y, 1);
		self.heuristic.update_done(self.step);
	}
	pub fn place_init(&mut self, x: i64, y: i64, tile: Tile) {
		self.move_list.push((x, y));
		self.update(x, y, tile);
	}
	pub fn place(&mut self, x: i64, y: i64) {
		self.move_list.push((x, y));
		let player = self.get_player();
		self.step += 1;
		self.update(x, y, player);
	}
	pub fn undo(&mut self) {
		let (x, y) = self.move_list.pop().unwrap();
		self.step -= 1;
		self.update(x, y, Tile::Empty);
	}

	pub fn tree_level(&self) -> usize { self.tree.lvl }
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