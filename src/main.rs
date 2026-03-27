mod game_map;

mod game_map {
    use std::io::empty;

    #[derive(Clone, Copy, PartialEq)]
    enum Tile {
        Empty = 0, X, O}
    enum Winner {None, X, O}

    const LEAF_LOG_SIZE: u64 = 2;
    const LEAF_SIZE: usize = 1 << LEAF_LOG_SIZE;

    struct LeafChunk {
        use_cnt: u64,
        tiles: [[Tile; LEAF_SIZE]; LEAF_SIZE],
    }

    impl LeafChunk {
        fn new() -> LeafChunk {
            LeafChunk {
                use_cnt: 0,
                tiles: [[Tile::Empty; LEAF_SIZE]; LEAF_SIZE]
            }
        }

        fn get_tile(&self, x: usize, y: usize) -> Tile {
            assert!(x < LEAF_SIZE);
            assert!(y < LEAF_SIZE);
            self.tiles[x][y]
        }

        fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
            self.use_cnt -= (self.tiles[x][y] != Tile::Empty) as u64;
            self.tiles[x][y] = tile;
            self.use_cnt += (tile != Tile::Empty) as u64;
        }

        fn is_empty(&self) -> bool { self.use_cnt == 0 }
    }

    const NODE_LOG_SIZE: u64 = 1;
    const NODE_SIZE: usize = 1 << NODE_LOG_SIZE;

    struct NodeChunk {
        use_cnt: u64,
        chd: [[Chunk; NODE_SIZE]; NODE_SIZE]
    }

    enum Chunk {
        Empty,
        Leaf(Box<LeafChunk>),
        Node(Box<NodeChunk>),
    }

    impl Clone for Chunk {
        fn clone(&self) -> Self {
            match self {
                Chunk::Empty => Chunk::Empty,
                _ => panic!("Attempted to clone non-empty chunk!")
            }
        }
    }

    type Level = u64;

    impl NodeChunk {
        fn new() -> NodeChunk {
            NodeChunk {
                use_cnt: 0,
                chd: std::array::repeat(std::array::repeat(Chunk::Empty)),
            }
        }

        fn find_chd(&self, x: &mut usize, y: &mut usize, lvl: Level) -> (usize, usize) {
            let scale = LEAF_LOG_SIZE + lvl * NODE_LOG_SIZE;
            let idx = *x >> scale;
            let idy = *y >> scale;
            *x -= idx << scale;
            *y -= idy << scale;
            assert!(idx < NODE_SIZE);
            assert!(idy < NODE_SIZE);
            (idx, idy)
        }

        fn get_tile(&self, mut x: usize, mut y: usize, lvl: Level) -> Tile {
            let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);
            match &self.chd[idx][idy] {
                Chunk::Empty => Tile::Empty,
                Chunk::Leaf(chunk) => chunk.get_tile(x, y),
                Chunk::Node(chunk) => chunk.get_tile(x, y, lvl - 1),
            }
        }

        fn set_tile(&mut self, mut x: usize, mut y: usize, tile: Tile, lvl: Level) {
            let (idx, idy) = self.find_chd(&mut x, &mut y, lvl);

            // Create child if empty
            if matches!(self.chd[idx][idy], Chunk::Empty)  {
                if tile == Tile::Empty { return }
                self.chd[idx][idy] = match lvl {
                    0 => Chunk::Leaf(Box::new(LeafChunk::new())),
                    _ => Chunk::Node(Box::new(NodeChunk::new())),
                };
                self.use_cnt += 1;
            }

            // Update child
            match &mut self.chd[idx][idy] {
                Chunk::Empty => unreachable!("Attempted to modify empty child node!"),
                Chunk::Leaf(chunk) => chunk.set_tile(x, y, tile),
                Chunk::Node(chunk) => chunk.set_tile(x, y, tile, lvl - 1),
            }

            // Delete child if empty
            if tile != Tile::Empty { return }
            if match &self.chd[idx][idy] {
                Chunk::Empty => unreachable!("Attempted to delete empty child node!"),
                Chunk::Leaf(chunk) => chunk.is_empty(),
                Chunk::Node(chunk) => chunk.is_empty(),
            } {
                self.chd[idx][idy] = Chunk::Empty;
                self.use_cnt -= 1;
            }
        }

        fn is_empty(&self) -> bool { self.use_cnt == 0 }
    }

    struct GameMap {
        off_x: i64,
        off_y: i64,
        lvl: Level,
        quad_tree: Box<NodeChunk>,
    }

    impl GameMap {
        fn get_tile(&self, mut x: i64, mut y: i64) -> Tile {
            x -= self.off_x;
            y -= self.off_y;
            let max_pos = 1 << (LEAF_LOG_SIZE + self.lvl * NODE_LOG_SIZE);
            if 0 < x || x >= max_pos || 0 < y || y >= max_pos {
                return Tile::Empty
            }
            self.quad_tree.get_tile(x as usize, y as usize, self.lvl)
        }

        fn set_tile(&mut self, mut x: i64, mut y: i64, tile: Tile) {
            x -= self.off_x;
            y -= self.off_y;
            loop {
                let max_pos = 1 << (LEAF_LOG_SIZE + self.lvl * NODE_LOG_SIZE);
                if 0 <= x && x < max_pos && 0 <= y && y < max_pos { break }
                let wrap_x = if x < 0 { NODE_SIZE - 1 } else { 0 };
                let wrap_y = if y < 0 { NODE_SIZE - 1 } else { 0 };
                let old_tree = std::mem::replace(&mut self.quad_tree, Box::new(NodeChunk::new()));
                self.quad_tree.chd[wrap_x][wrap_y] = Chunk::Node(old_tree);
                self.lvl += 1;
            }
            self.quad_tree.set_tile(x as usize, y as usize, tile, self.lvl)
        }
    }
}

fn main() {
    println!("Hello, world!");
}
