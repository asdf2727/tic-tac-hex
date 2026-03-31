use super::*;
use chunk::Chunk;
use params::NODE_LOG_SIZE;

pub const NODE_SIZE: usize = 1 << NODE_LOG_SIZE;

pub struct QuadNode {
	pub chd: [[QuadChd; NODE_SIZE]; NODE_SIZE],
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

pub type Level = usize;

impl QuadNode {
	pub fn new() -> QuadNode {
		QuadNode {
			chd: std::array::repeat(std::array::repeat(QuadChd::Empty)),
		}
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

	pub fn try_get_chunk(&self, mut x: usize, mut y: usize, lvl: Level) -> Option<&Chunk> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
		match &self.chd[idx][idy] {
			QuadChd::Empty => None,
			QuadChd::Leaf(chunk) => Some(chunk),
			QuadChd::Node(chunk) => chunk.try_get_chunk(x, y, lvl - 1),
		}
	}
	pub fn try_get_chunk_mut(&mut self, mut x: usize, mut y: usize, lvl: Level) -> Option<&mut Chunk> {
		let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
		match &mut self.chd[idx][idy] {
			QuadChd::Empty => None,
			QuadChd::Leaf(chunk) => Some(chunk),
			QuadChd::Node(chunk) => chunk.try_get_chunk_mut(x, y, lvl - 1),
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

	pub fn is_empty(&self) -> bool {
		self.chd.iter().flatten().all(|chd| matches!(chd, QuadChd::Empty))
	}
	pub fn clean(&mut self) -> bool {
		for idx in 0..NODE_SIZE {
			for idy in 0..NODE_SIZE {
				if match &mut self.chd[idx][idy] {
					QuadChd::Empty => false,
					QuadChd::Leaf(chunk) => chunk.is_empty(),
					QuadChd::Node(chunk) => chunk.clean(),
				} {
					self.chd[idx][idy] = QuadChd::Empty;
				}
			}
		}
		self.is_empty()
	}
}