#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Tile {Empty = 0, X = 1, O = 2}

pub const TILE_BITS: usize = 2;

impl Tile {
	pub fn to_char(&self) -> char {
		match self {
			Tile::Empty => ' ',
			Tile::X => 'X',
			Tile::O => 'O',
		}
	}
}

impl std::fmt::Display for Tile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_char())
	}
}