use super::{*};
use chunk::Chunk;
use params::NODE_LOG_SIZE;

pub const NODE_SIZE: usize = 1 << NODE_LOG_SIZE;

pub struct QuadNode {
	pub(super) chd: [[QuadChd; NODE_SIZE]; NODE_SIZE],
}

pub enum QuadChd {
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

pub type Level = u64;

impl QuadNode {
	pub fn new() -> QuadNode {
		QuadNode {
			chd: std::array::repeat(std::array::repeat(QuadChd::Empty)),
		}
	}

	pub fn wrap(&mut self, wrap_x: usize, wrap_y: usize) {
		let old_root = std::mem::replace(self, QuadNode::new());
		self.chd[wrap_x][wrap_y] = QuadChd::Node(Box::new(old_root));
	}

	fn find_chd(&self, x: &mut usize, y: &mut usize, lvl: Level) -> (usize, usize) {
		let scale = (lvl - 1) * NODE_LOG_SIZE;
		let idx = *x >> scale;
		let idy = *y >> scale;
		*x -= idx << scale;
		*y -= idy << scale;
		debug_assert!(idx < NODE_SIZE);
		debug_assert!(idy < NODE_SIZE);
		(idx, idy)
	}

	pub fn try_get_chunk_mut(&mut self, mut x: usize, mut y: usize, lvl: Level) -> Option<&mut Chunk> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);

		if match &self.chd[idx][idy] {
			QuadChd::Empty => false,
			QuadChd::Leaf(chunk) => chunk.is_empty(),
			QuadChd::Node(chunk) => chunk.is_empty(),
		} {
			self.chd[idx][idy] = QuadChd::Empty;
		}

		match &mut self.chd[idx][idy] {
			QuadChd::Empty => None,
			QuadChd::Leaf(chunk) => Some(chunk),
			QuadChd::Node(chunk) => chunk.try_get_chunk_mut(x, y, lvl - 1),
		}
	}
	pub fn try_get_chunk(&mut self, x: usize, y: usize, lvl: Level) -> Option<&Chunk> {
		Some(self.try_get_chunk_mut(x, y, lvl)?)
	}

	pub fn try_get_chunk_const(&self, mut x: usize, mut y: usize, lvl: Level) -> Option<&Chunk> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
		match &self.chd[idx][idy] {
			QuadChd::Empty => None,
			QuadChd::Leaf(chunk) => Some(&chunk),
			QuadChd::Node(chunk) => chunk.try_get_chunk_const(x, y, lvl - 1),
		}
	}

	pub fn get_chunk_mut(&mut self, mut x: usize, mut y: usize, lvl: Level) -> &mut Chunk {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);

		if matches!(self.chd[idx][idy], QuadChd::Empty)  {
			self.chd[idx][idy] = match lvl {
				0 => unreachable!("Attempted to split leaf node!"),
				1 => QuadChd::Leaf(Box::new(Chunk::new())),
				_ => QuadChd::Node(Box::new(QuadNode::new())),
			};
		}

		match &mut self.chd[idx][idy] {
			QuadChd::Empty => unreachable!("Attempted to access empty child node!"),
			QuadChd::Leaf(chunk) => chunk.as_mut(),
			QuadChd::Node(chunk) => chunk.get_chunk_mut(x, y, lvl - 1),
		}
	}
	pub fn get_chunk(&mut self, x: usize, y: usize, lvl: Level) -> &Chunk {
		self.get_chunk_mut(x, y, lvl)
	}

	fn build_neighbours(&self, x: usize, y: usize, lvl: Level) -> Option<Neighbours> {
		let Some(mid) = self.try_get_chunk_const(x, y, lvl) else { return None; };
		Some(Neighbours {
			chunks: [[
				self.try_get_chunk_const(x - 1, y - 1, lvl),
				self.try_get_chunk_const(x - 1, y, lvl),
				self.try_get_chunk_const(x - 1, y + 1, lvl),
			], [
				self.try_get_chunk_const(x, y - 1, lvl),
				Some(mid),
				self.try_get_chunk_const(x, y + 1, lvl),
			], [
				self.try_get_chunk_const(x + 1, y - 1, lvl),
				self.try_get_chunk_const(x + 1, y, lvl),
				self.try_get_chunk_const(x + 1, y + 1, lvl),
			]]
		})
	}

	pub fn get_neighbours(&self, mut x: usize, mut y: usize, lvl: Level) -> Option<Neighbours> {
		let size = 1 << (lvl * NODE_LOG_SIZE);
		if x == size / 2 - 1 || x == size / 2 || y == size / 2 - 1 || y == size / 2 {
			return self.build_neighbours(x, y, lvl);
		}
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
		let QuadChd::Node(chunk) = &self.chd[idx][idy] else { return None; };
		chunk.get_neighbours(x, y, lvl - 1)
	}

	pub fn get_all_neighbours(&self, lvl: Level) -> Vec<Box<Neighbours>> {
		let mut ans: Vec<Box<Neighbours>> = Vec::new();
		let size = 1 << (lvl * NODE_LOG_SIZE);
		self.chd.iter().flatten().for_each(|chd| match chd {
			QuadChd::Empty => {},
			QuadChd::Leaf(_) => unreachable!("Attempted to get all neighbours of a leaf!"),
			QuadChd::Node(chunk) => {
				ans.extend(chunk.get_all_neighbours(lvl - 1).into_iter());
			},
		});

		for i in 1..size - 1 {
			if let Some(neigh) = self.get_neighbours(size / 2 - 1, i, lvl) {
				ans.push(Box::from(neigh));
			}
			if let Some(neigh) = self.get_neighbours(size / 2, i, lvl) {
				ans.push(Box::from(neigh));
			}
			if let Some(neigh) = self.get_neighbours(i, size / 2 - 1, lvl) {
				ans.push(Box::from(neigh));
			}
			if let Some(neigh) = self.get_neighbours(i, size / 2, lvl) {
				ans.push(Box::from(neigh));
			}
		}
		ans
	}

	pub fn is_empty(&self) -> bool {
		self.chd.iter().flatten().all(|chd| matches!(chd, QuadChd::Empty))
	}
}