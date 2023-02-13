// Totally not stolen from central_program

use crate::tile::{Colour, Tile};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DFS {
    Visited,
    Visiting,
    Unvisited,
}

#[derive(Debug)]
pub struct Board {
    size: usize,
    board: Vec<Colour>,
}

impl Board {
    pub fn new(size: u8) -> Self {
        Self {
            size: size as usize,
            board: vec![Colour::Empty; size.pow(2) as usize],
        }
    }

    // Create board from bot output string. Ex: "...|B.B|.W.|"
    pub fn from(compressed: &str) -> Self {
        let board = compressed
            .trim()
            .chars()
            .filter(|&c| c != '|')
            .map(|c| match c {
                'B' => Colour::Black,
                'W' => Colour::White,
                '.' => Colour::Empty,
                _ => panic!("Incorrect bot output character`{}`", c),
            })
            .collect::<Vec<Colour>>();

        Self {
            size: (board.len() as f64).sqrt() as usize,
            board,
        }
    }

    fn tile_to_index(&self, tile: Tile) -> Option<usize> {
        match tile {
            Tile::Tile(r, c) => Some((r as usize) * self.size + c as usize),
            Tile::Edge1 | Tile::Edge2 | Tile::Invalid => None,
        }
    }

    pub fn get_tile(&self, tile: Tile) -> Option<Colour> {
        self.tile_to_index(tile)
            .and_then(|idx| self.board.get(idx).copied())
    }

    pub fn set_move(&mut self, mv: &str, color: Colour) {
        let index = self.move_to_index(mv);
        self.board[index] = color
    }

    // Returns true when the specified tile is empty
    pub fn is_valid_move(&self, mv: &str) -> bool {
        self.board[self.move_to_index(mv)] == Colour::Empty
    }

    // Returns the color of the player who won, empty otherwise
    pub fn has_win(&self) -> Colour {
        if self.is_black_win() {
            Colour::Black
        } else if self.is_white_win() {
            Colour::White
        } else {
            Colour::Empty
        }
    }

    // Returns an array of all indicies adjacent to a given hex. That's 2-5 indicies
    fn get_adj(&self, row: usize, column: usize) -> Vec<usize> {
        let r = row as isize;
        let c = column as isize;
        let s = self.size as isize;

        let a = [
            (r, c - 1),
            (r + 1, c - 1),
            (r - 1, c),
            (r + 1, c),
            (r - 1, c + 1),
            (r, c + 1),
        ];

        a.into_iter()
            .filter(|(r, c)| 0 <= *r && *r < s && 0 <= *c && *c < s)
            .map(|(r, c)| self.coord_to_index(r as usize, c as usize))
            .collect()
    }

    fn coord_to_index(&self, r: usize, c: usize) -> usize {
        r * self.size + c
    }

    fn index_to_coord(&self, i: usize) -> Option<(usize, usize)> {
        if i < self.board.len() {
            Some((i / self.size, i % self.size))
        } else {
            None
        }
    }

    // Converts a move (ex: "a1") to the board's index
    fn move_to_index(&self, mv: &str) -> usize {
        let r = mv.chars().nth(0).unwrap() as usize - 97; // First letter
        let c = mv[1..].parse::<usize>().unwrap() - 1; // All following digits

        self.coord_to_index(r, c)
    }

    fn is_black_win(&self) -> bool {
        let mut dfs_tree = vec![DFS::Unvisited; self.size.pow(2)];

        for start_col in 0..self.size {
            let start = self.coord_to_index(0, start_col);

            if self.board[start] == Colour::Black && dfs_tree[start] == DFS::Unvisited {
                for adj in self.get_adj(0, start_col).into_iter() {
                    if self.board[adj] == Colour::Black {
                        dfs_tree[start] = DFS::Visited;

                        if self.has_path(adj, Colour::Black, &mut dfs_tree) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_white_win(&self) -> bool {
        let mut dfs_tree = vec![DFS::Unvisited; self.size.pow(2)];

        for start_row in 0..self.size {
            let start = self.coord_to_index(start_row, 0);

            if self.board[start] == Colour::White && dfs_tree[start] == DFS::Unvisited {
                for adj in self.get_adj(start_row, 0).into_iter() {
                    if self.board[adj] == Colour::White {
                        dfs_tree[start] = DFS::Visited;

                        if self.has_path(adj, Colour::White, &mut dfs_tree) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn has_path(&self, from: usize, color: Colour, dfs_tree: &mut Vec<DFS>) -> bool {
        let (r, c) = self.index_to_coord(from).unwrap();

        if color == Colour::White && c == self.size - 1
            || color == Colour::Black && r == self.size - 1
        {
            return true;
        }

        dfs_tree[from] = DFS::Visiting;
        for adj in self.get_adj(r, c).into_iter() {
            if dfs_tree[adj] == DFS::Unvisited && self.board[adj] == color {
                if self.has_path(adj, color, dfs_tree) {
                    return true;
                }
            }
        }
        false
    }
}

impl std::fmt::Display for Board {
    // Example output:
    // B . . .
    //  . B W .
    //   . . B .
    //    W . W B
    // ------------------
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.size {
            write!(f, "{}", " ".repeat(r))?;

            for c in 0..self.size {
                match self.board[self.coord_to_index(r, c)] {
                    Colour::Black => write!(f, "B ")?,
                    Colour::White => write!(f, "W ")?,
                    Colour::Empty => write!(f, ". ")?,
                }
            }
            write!(f, "\n")?;
        }

        write!(f, "{}", "-".repeat(18))
    }
}

#[cfg(test)]
mod board_testing {
    use super::*;

    #[test]
    fn setters_getters() {
        let mut board = Board::new(4);

        for r in 0..4 {
            for c in 0..4 {
                assert_eq!(board.get(r, c), Some(Colour::Empty));
            }
        }

        board.set(0, 3, Colour::White);
        assert_eq!(board.get(0, 3), Some(Colour::White));

        board.set(3, 2, Colour::Black);
        assert_eq!(board.get(3, 2), Some(Colour::Black));

        board.set(3, 2, Colour::Empty);
        assert_eq!(board.get(3, 2), Some(Colour::Empty));
    }

    #[test]
    fn adjacent_tiles() {
        let mut board = Board::new(5);
        {
            let r = 2;
            let c = 2;

            let adjs = board.get_adj(r, c);
            let expected = vec![7, 8, 11, 13, 16, 17];

            assert_eq!(adjs.iter().sum::<usize>(), expected.iter().sum::<usize>());
        }
        {
            let r = 1;
            let c = 4;

            let mut adjs = board.get_adj(r, c);
            let mut expected = vec![4, 8, 13, 14];

            assert_eq!(adjs.iter().sum::<usize>(), expected.iter().sum::<usize>());
        }
        {
            let r = 4;
            let c = 4;

            let mut adjs = board.get_adj(r, c);
            let mut expected = vec![19, 23];

            assert_eq!(adjs.iter().sum::<usize>(), expected.iter().sum::<usize>());
        }
    }

    #[test]
    fn drawing() {
        let mut board = Board::new(4);

        board.set(0, 0, Colour::Black);
        board.set(1, 1, Colour::Black);
        board.set(2, 2, Colour::Black);
        board.set(3, 3, Colour::Black);

        board.set(3, 0, Colour::White);
        board.set(3, 2, Colour::White);
        board.set(1, 2, Colour::White);

        let expected = "B . . . \n . B W . \n  . . B . \n   W . W B \n------------------";

        assert_eq!(format!("{}", board), expected);

        board.set(0, 1, Colour::Black);
        board.set(0, 2, Colour::Black);
        board.set(0, 3, Colour::Black);

        let expected2 = "B B B B \n . B W . \n  . . B . \n   W . W B \n------------------";

        assert_eq!(format!("{}", board), expected2);
    }

    #[test]
    fn check_win() {
        {
            let mut board = Board::new(4);
            board.set(0, 0, Colour::White);
            board.set(0, 1, Colour::White);
            board.set(0, 2, Colour::White);
            board.set(0, 3, Colour::White);

            assert_eq!(board.has_win(), Colour::White);
        }
        {
            let mut board = Board::new(4);
            board.set(0, 0, Colour::Black);
            board.set(1, 0, Colour::Black);
            board.set(2, 0, Colour::Black);
            board.set(3, 0, Colour::Black);

            assert_eq!(board.has_win(), Colour::Black);
        }
        {
            let mut board = Board::new(4);
            board.set(0, 0, Colour::Black);
            board.set(0, 1, Colour::Black);
            board.set(0, 2, Colour::Black);
            board.set(0, 3, Colour::Black);

            board.set(1, 0, Colour::White);
            board.set(2, 0, Colour::White);
            board.set(3, 0, Colour::White);

            assert_eq!(board.has_win(), Colour::Empty);
        }
        {
            let mut board = Board::new(4);
            board.set(2, 0, Colour::White);
            board.set(2, 1, Colour::White);
            board.set(1, 2, Colour::White);
            board.set(0, 3, Colour::White);

            board.set(1, 1, Colour::Black);
            board.set(2, 2, Colour::Black);
            board.set(1, 3, Colour::Black);
            board.set(3, 2, Colour::Black);

            assert_eq!(board.has_win(), Colour::White);
        }
        {
            let mut board = Board::new(4);
            board.set(3, 0, Colour::White);
            board.set(2, 1, Colour::White);
            board.set(1, 2, Colour::White);
            board.set(0, 3, Colour::White);
            board.set(1, 3, Colour::White);
            board.set(2, 3, Colour::White);
            board.set(3, 3, Colour::White);

            board.set(1, 1, Colour::Black);
            board.set(3, 2, Colour::Black);
            board.set(0, 0, Colour::Black);
            board.set(0, 1, Colour::Black);
            board.set(1, 1, Colour::Black);
            board.set(2, 0, Colour::Black);
            board.set(3, 1, Colour::Black);

            assert_eq!(board.has_win(), Colour::White);

            board.set(2, 1, Colour::Black);
            assert_eq!(board.has_win(), Colour::Black);

            board.set(2, 1, Colour::Empty);
            assert_eq!(board.has_win(), Colour::Empty);
        }
    }
}
