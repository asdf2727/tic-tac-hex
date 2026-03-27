use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {Empty, X, O}
impl Display for Tile {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Tile::Empty => write!(f, " "),
			Tile::X => write!(f, "X"),
			Tile::O => write!(f, "O"),
		}
	}
}

use crate::game_params::WIN_LEN;

pub(crate) type CheckResult = Result<(), ()>;
pub trait LineChecker {
	fn on_line(&mut self, line: &[Tile; WIN_LEN]) -> CheckResult;
}

const CHUNK_LOG_SIZE: u64 = 2;
const CHUNK_SIZE: usize = 1 << CHUNK_LOG_SIZE;

struct Chunk {
	use_cnt: u64,
	tiles: [[Tile; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	fn new() -> Chunk {
		Chunk {
			use_cnt: 0,
			tiles: [[Tile::Empty; CHUNK_SIZE]; CHUNK_SIZE]
		}
	}

	fn get_tile(&self, x: usize, y: usize) -> Tile {
		debug_assert!(x < CHUNK_SIZE);
		debug_assert!(y < CHUNK_SIZE);
		self.tiles[x][y]
	}

	fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
		self.use_cnt -= (self.tiles[x][y] != Tile::Empty) as u64;
		self.tiles[x][y] = tile;
		self.use_cnt += (tile != Tile::Empty) as u64;
	}

	fn is_empty(&self) -> bool { self.use_cnt == 0 }
}

struct NeighbourChunks<'a> {
	chunks: [[Option<&'a Chunk>; 3]; 3]
}

impl NeighbourChunks<'_> {
	fn get_tile(&self, mut x: i64, mut y: i64) -> Tile {
		let mut idx = 1;
		let mut idy = 1;
		if x < 0 { idx -= 1; x -= CHUNK_SIZE as i64 }
		else if x > CHUNK_SIZE as i64 { idx += 1; x += CHUNK_SIZE as i64 }
		if y < 0 { idy -= 1; y -= CHUNK_SIZE as i64 }
		else if y > CHUNK_SIZE as i64 { idy += 1; y += CHUNK_SIZE as i64 }
		match self.chunks[idx][idy] {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(x as usize, y as usize)
		}
	}

	fn run_line_check<T: LineChecker>(&self, checker: &mut T) -> CheckResult {
		debug_assert!(CHUNK_SIZE > WIN_LEN);
		let min_x: i64 = if self.chunks[0][1].is_some() { 0 } else { 1 - WIN_LEN as i64 };
		let min_y: i64 = if self.chunks[1][0].is_some() { 0 } else { 1 - WIN_LEN as i64 };
		for x in min_x..CHUNK_SIZE as i64 {
			let start_y = if x < 0 && self.chunks[0][0].is_some() { 0 } else { min_y };
			for y in start_y..CHUNK_SIZE as i64 {
				checker.on_line(&std::array::from_fn(|i| self.get_tile(x, y + i as i64)))?;
				checker.on_line(&std::array::from_fn(|i| self.get_tile(x + i as i64, y)))?;
				checker.on_line(&std::array::from_fn(|i| self.get_tile(x + i as i64, y + i as i64)))?;
			}
		}
		Ok(())
	}
}

const NODE_LOG_SIZE: u64 = 1;
const NODE_SIZE: usize = 1 << NODE_LOG_SIZE;

struct QuadNode {
	use_cnt: u64,
	chd: [[QuadChd; NODE_SIZE]; NODE_SIZE],
}

enum QuadChd {
	Empty,
	Leaf(Box<Chunk>),
	Node(Box<QuadNode>),
}

impl Clone for QuadChd {
	fn clone(&self) -> Self {
		match self {
			QuadChd::Empty => QuadChd::Empty,
			_ => panic!("Attempted to clone non-empty chunk!")
		}
	}
}

type Level = u64;

type NodeCheckResult = Result<Vec<(usize, usize)>, ()>;

impl QuadNode {
	fn new() -> QuadNode {
		QuadNode {
			use_cnt: 0,
			chd: std::array::repeat(std::array::repeat(QuadChd::Empty)),
		}
	}

	fn find_chd(&self, x: &mut usize, y: &mut usize, lvl: Level) -> (usize, usize) {
		let scale = lvl * NODE_LOG_SIZE;
		let idx = *x >> scale;
		let idy = *y >> scale;
		*x -= idx << scale;
		*y -= idy << scale;
		debug_assert!(idx < NODE_SIZE);
		debug_assert!(idy < NODE_SIZE);
		(idx, idy)
	}

	fn try_get_chunk_mut(&mut self, mut x: usize, mut y: usize, lvl: Level) -> Result<&mut Chunk, ()> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);

		if match &self.chd[idx][idy] {
			QuadChd::Empty => false,
			QuadChd::Leaf(chunk) => chunk.is_empty(),
			QuadChd::Node(chunk) => chunk.is_empty(),
		} {
			self.chd[idx][idy] = QuadChd::Empty;
			self.use_cnt -= 1;
		}

		Ok(match &mut self.chd[idx][idy] {
			QuadChd::Empty => return Err(()),
			QuadChd::Leaf(chunk) => chunk.as_mut(),
			QuadChd::Node(chunk) => chunk.try_get_chunk_mut(x, y, lvl - 1)?,
		})
	}
	fn try_get_chunk(&mut self, x: usize, y: usize, lvl: Level) -> Result<&Chunk, ()> {
		Ok(self.try_get_chunk_mut(x, y, lvl)?)
	}

	fn try_get_chunk_const(&self, mut x: usize, mut y: usize, lvl: Level) -> Result<&Chunk, ()> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
		Ok(match &self.chd[idx][idy] {
			QuadChd::Empty => return Err(()),
			QuadChd::Leaf(chunk) => chunk.as_ref(),
			QuadChd::Node(chunk) => chunk.try_get_chunk_const(x, y, lvl - 1)?,
		})
	}

	fn get_chunk_mut(&mut self, mut x: usize, mut y: usize, lvl: Level) -> &mut Chunk {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);

		if matches!(self.chd[idx][idy], QuadChd::Empty)  {
			self.chd[idx][idy] = match lvl {
				0 => QuadChd::Leaf(Box::new(Chunk::new())),
				_ => QuadChd::Node(Box::new(QuadNode::new())),
			};
			self.use_cnt += 1;
		}

		match &mut self.chd[idx][idy] {
			QuadChd::Empty => unreachable!("Attempted to access empty child node!"),
			QuadChd::Leaf(chunk) => chunk.as_mut(),
			QuadChd::Node(chunk) => chunk.get_chunk_mut(x, y, lvl - 1),
		}
	}
	fn get_chunk(&mut self, x: usize, y: usize, lvl: Level) -> &Chunk {
		self.get_chunk_mut(x, y, lvl)
	}

	fn is_empty(&self) -> bool { self.use_cnt == 0 }

	fn run_line_check<T: LineChecker>(&mut self, checker: &mut T, lvl: Level) -> NodeCheckResult {
		let chd_size = 1 << lvl * NODE_LOG_SIZE;
		let size = chd_size << NODE_LOG_SIZE;
		let mut leaves = Vec::new();

		for idx in 0..NODE_SIZE {
			for idy in 0..NODE_SIZE {
				let mut chd_leaves = Vec::new();
				match &mut self.chd[idx][idy] {
					QuadChd::Empty => {},
					QuadChd::Leaf(_) => chd_leaves.push((0, 0)),
					QuadChd::Node(chunk) => chd_leaves = chunk.run_line_check(checker, lvl - 1)?,
				};

				for (mut x, mut y) in chd_leaves {
					x += chd_size * idx;
					y += chd_size * idy;
					if x == 0 || x == size - 1 || y == 0 || y == size - 1 { leaves.push((x, y)); }
					for i in 0..3 {
						for j in 0..3 {
							let _ = self.try_get_chunk(x + i - 1, y + j - 1, lvl); // For cleanup
						}
					}
					NeighbourChunks {
						chunks: std::array::from_fn(|i|
							std::array::from_fn(|j|
								self.try_get_chunk_const(x + i - 1, y + j - 1, lvl).ok()))
					}.run_line_check(checker)?
				}
			}
		}
		Ok(leaves)
	}
}

pub struct GameMap {
	off_x: usize,
	off_y: usize,
	lvl: Level,
	quad_tree: QuadNode,
}

impl GameMap {
	pub fn new() -> GameMap {
		GameMap {
			off_x: 0,
			off_y: 0,
			lvl: 0,
			quad_tree: QuadNode::new(),
		}
	}

	fn get_chunk(&mut self, x: usize, y: usize) -> Option<&Chunk> {
		let max_pos = 1 << (self.lvl + 1) * NODE_LOG_SIZE;
		if x >= max_pos || y >= max_pos {
			return None
		}
		match self.quad_tree.try_get_chunk(x, y, self.lvl) {
			Err(()) => None,
			Ok(chunk) => Some(chunk)
		}
	}

	pub fn get_tile(&mut self, x: i64, y: i64) -> Tile {
		match self.get_chunk((x >> CHUNK_LOG_SIZE) as usize + self.off_x,
		                     (y >> CHUNK_LOG_SIZE) as usize + self.off_y) {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(x as usize & (CHUNK_SIZE - 1), y as usize & (CHUNK_SIZE - 1))
		}
	}

	pub fn set_tile(&mut self, x: i64, y: i64, tile: Tile) {
		let mut chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let mut chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		loop {
			let max_pos = 1 << (self.lvl + 1) * NODE_LOG_SIZE;
			if 0 <= chk_x && chk_x < max_pos as i64 && 0 <= chk_y && chk_y < max_pos as i64 { break }

			let wrap_x = if chk_x < 0 { NODE_SIZE - 1 } else { 0 };
			let wrap_y = if chk_y < 0 { NODE_SIZE - 1 } else { 0 };
			let old_tree = std::mem::replace(&mut self.quad_tree, QuadNode::new());
			self.quad_tree.chd[wrap_x][wrap_y] = QuadChd::Node(Box::new(old_tree));
			self.quad_tree.use_cnt += 1;

			let shift_x = max_pos * wrap_x;
			let shift_y = max_pos * wrap_y;
			self.off_x += shift_x;
			self.off_y += shift_y;
			chk_x += shift_x as i64;
			chk_y += shift_y as i64;

			self.lvl += 1;
		}

		self.quad_tree.get_chunk_mut(chk_x as usize, chk_y as usize, self.lvl)
			.set_tile(x as usize & (CHUNK_SIZE - 1), y as usize & (CHUNK_SIZE - 1), tile)
	}

	fn run_line_check<T: LineChecker>(&mut self, checker: &mut T) -> CheckResult {
		let leaves = self.quad_tree.run_line_check(checker, self.lvl)?;
		for (x, y) in leaves {
			for i in 0..3 {
				for j in 0..3 {
					let _ = self.get_chunk(x + i - 1, y + j - 1); // For cleanup
				}
			}
			NeighbourChunks {
				chunks: std::array::from_fn(|i|
					std::array::from_fn(|j| {
						let idx = x + i - 1;
						let idy = y + j - 1;
						let size = 1 << (NODE_LOG_SIZE * (self.lvl + 1));
						if 0 >= idx || idx >= size || 0 >= idy || idy >= size { return None }
						self.quad_tree.try_get_chunk_const(idx, idy, self.lvl).ok()
					}))
			}.run_line_check(checker)?
		}
		Ok(())
	}
}

impl Display for GameMap {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let max_pos: i64 = 1 << (CHUNK_LOG_SIZE + (self.lvl + 1) * NODE_LOG_SIZE);
		for x in 0i64 - (CHUNK_SIZE * self.off_x) as i64..max_pos - (CHUNK_SIZE * self.off_x) as i64 {
			write!(f, "{}", " ".repeat((max_pos - x) as usize))?;
			for y in 0i64 - (CHUNK_SIZE * self.off_y) as i64..max_pos - (CHUNK_SIZE * self.off_y) as i64 {
				let mut delim: char = ' ';
				if y - (x >> 1) == 0 && (x & 1) != 0 { delim = '|'; }
				if x == 0 { delim = '-'; }

				let tile = match self.quad_tree.try_get_chunk_const(((x >> CHUNK_LOG_SIZE) + self.off_x as i64) as usize,
				                                                    ((y >> CHUNK_LOG_SIZE) + self.off_y as i64) as usize, self.lvl) {
					Err(()) => Tile::Empty,
					Ok(chunk) => chunk.get_tile(x as usize & (CHUNK_SIZE - 1), y as usize & (CHUNK_SIZE - 1))
				};

				write!(f, "{}{}", tile, delim)?
			}
			writeln!(f)?
		}
		Ok(())
	}
}