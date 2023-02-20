use std::collections::HashMap;

use crate::{
    board::Board,
    tile::{Colour, PieceState, Tile},
};

struct Explore<'a> {
    best_move: Tile,
    moves: &'a HashMap<Tile, f32>,
    board: &'a Board,
    minimum: f32,
}

pub fn explore_other_moves(board: &Board, moves: HashMap<Tile, f32>, best_move: Tile) -> Tile {
    let mut exp = Explore {
        best_move,
        moves: &moves,
        board,
        minimum: Default::default(),
    };
    exp.update_minimum();
    exp.minimum += 1080.;
    for i in 0..exp.board.size {
        for j in 0..exp.board.size {
            let tile = Tile::Valid(i, j);
            if let Some(PieceState::Colour(c)) = exp.board.get_tile(tile) {
                if let Some(&val) = exp.moves.get(&tile) {
                    if val < exp.minimum {
                        match c {
                            Colour::Black => exp.handle_black(i, j),
                            Colour::White => exp.handle_white(i, j),
                        }
                    }
                }
            }
        }
    }

    exp.best_move
}

fn in_range(i: i8, j: i8, i_min: i8, i_max: i8, j_min: i8, j_max: i8) -> bool {
    i >= i_min && i < i_max && j >= j_min && j < j_max
}

impl Explore<'_> {
    fn get_fld(&self, tile: Tile) -> PieceState {
        match tile {
            Tile::Valid(i, _) if i < 0 || i >= self.board.size => PieceState::Colour(Colour::Black),
            Tile::Valid(_, j) if j < 0 || j >= self.board.size => PieceState::Colour(Colour::White),
            Tile::Valid(i, j) => self
                .board
                .get(i, j)
                .expect(format!("valid row {i} and col {j}").as_str()),
            Tile::Edge1 | Tile::Edge2 | Tile::Invalid => panic!("passed an invalid field"),
        }
    }

    fn can_connect(&self, n: i8, m: i8, colour: Colour, is_row: bool) -> i8 {
        let min = if is_row { m } else { n };
        let max = if is_row { self.board.size - 1 - n } else { m };
        let dir: i32 = if is_row { 1 } else { -1 };
        for i in 0..self.board.size {
            for j in (min..=max).step_by(dir as usize) {
                let x = if is_row { i as i8 } else { j };
                let y = if is_row { j } else { i as i8 };
                if (x - n).abs() == (y - m).abs() {
                    if let Some(PieceState::Colour(_)) = self.board.get(x, y) {
                        return 2;
                    }
                }
            }
        }
        let val = colour.val();
        let p = if is_row {
            Tile::Valid(n, m - val)
        } else {
            Tile::Valid(n - val, m)
        };
        let q = if is_row {
            Tile::Valid(n - val, m + val)
        } else {
            Tile::Valid(n + val, m - val)
        };
        let r = if is_row {
            Tile::Valid(n + val, m - 2 * val)
        } else {
            Tile::Valid(n - 2 * val, m + val)
        };
        if self.board.get_tile(p) == Some(PieceState::Colour(colour.opponent())) {
            return 0;
        }
        if self.board.get_tile(q) == Some(PieceState::Colour(colour.opponent())) {
            if self.get_fld(r) == PieceState::Colour(colour.opponent()) {
                return 0;
            }
            return -1;
        }
        if self.get_fld(r) == PieceState::Colour(colour.opponent()) {
            return -2;
        }
        return 1;
    }

    fn can_connect_far_border(&self, n: i8, m: i8, colour: Colour) -> i8 {
        match colour {
            Colour::Black if 2 * n < self.board.size - 1 => self.can_connect(n, m, colour, true),
            Colour::White if 2 * n < self.board.size - 1 => self.can_connect(n, m, colour, false),
            Colour::Black => self.can_connect(n, m, colour, false),
            Colour::White => self.can_connect(n, m, colour, true),
        }
    }

    fn upd(&mut self, i: i8, j: i8) {
        self.best_move = Tile::Valid(i, j);
        self.update_minimum();
    }

    fn upd_inc_i(&mut self, mut cc: i8, inc: i8, i: i8, j: i8) {
        let mut best_i = i;
        if cc < 2 {
            if cc < -1 {
                best_i += inc;
                cc += 1;
            }
            self.best_move = Tile::Valid(best_i, j + (inc * cc));
            self.update_minimum();
        }
    }

    fn upd_inc_j(&mut self, mut cc: i8, inc: i8, i: i8, j: i8) {
        let mut best_j = j;
        if cc < 2 {
            if cc < -1 {
                best_j += inc;
                cc += 1;
            }
            self.best_move = Tile::Valid(i + (inc * cc), best_j);
            self.update_minimum();
        }
    }

    fn has_enemy(&self, i: i8, j: i8, colour: Colour) -> bool {
        matches!(self.board.get(i, j), Some(PieceState::Colour(c)) if c == colour.opponent())
    }

    fn empty(&self, idx: [(i8, i8); 4]) -> bool {
        idx.iter()
            .all(|i| matches!(self.board.get(i.0, i.1), Some(PieceState::Empty)))
    }

    fn update_minimum(&mut self) {
        self.minimum = *self.moves.get(&self.best_move).expect(
            format!(
                "best move {} to be in the moves map. {:?}",
                self.best_move, &self.moves
            )
            .as_str(),
        );
    }
    fn handle_black(&mut self, i: i8, j: i8) {
        let size = self.board.size;
        let c = Colour::White;
        if in_range(i, j, 4, size - 1, 1, 3) && self.has_enemy(i - 1, j + 2, c) {
            let cc = self.can_connect_far_border(i - 1, j + 2, c.opponent());
            self.upd_inc_i(cc, -1, i, j);
        }
        if in_range(i, j, 1, size - 1, 0, 1)
            && self.has_enemy(i - 1, j - 2, c)
            && self.empty([(i - 1, j), (i - 1, j + 1), (i + 1, j), (i, j + 1)])
        {
            self.upd(i, j);
        }
        if in_range(i, j, 1, size - 4, size - 3, size - 1) && self.has_enemy(i + 1, j - 2, c) {
            let cc = self.can_connect_far_border(i + 1, j - 2, c.opponent());
            self.upd_inc_i(cc, 1, i, j);
        }
        if in_range(i, j, 1, size - 1, size - 1, size)
            && self.has_enemy(i + 1, j - 2, c)
            && self.empty([(i + 1, j), (i + 1, j - 1), (i - 1, j), (i, j - 1)])
        {
            self.upd(i, j);
        }
    }
    fn handle_white(&mut self, i: i8, j: i8) {
        let size = self.board.size;
        let c = Colour::White;
        if in_range(i, j, 1, 3, 4, size - 1) && self.has_enemy(i + 2, j - 1, c) {
            let cc = self.can_connect_far_border(i + 2, j - 1, c.opponent());
            self.upd_inc_j(cc, -1, i, j);
        }
        if in_range(i, j, 0, 1, 1, size - 1)
            && self.has_enemy(i + 2, j - 1, c)
            && self.empty([(i, j - 1), (i + 1, j - 1), (i + 1, j), (i, j + 1)])
        {
            self.upd(i, j);
        }
        if in_range(i, j, size - 3, size - 1, 1, size - 4) && self.has_enemy(i - 2, j + 1, c) {
            let cc = self.can_connect_far_border(i - 2, j + 1, c.opponent());
            self.upd_inc_j(cc, 1, i, j);
        }
        if in_range(i, j, size - 1, size, 1, size - 1)
            && self.has_enemy(i - 2, j + 1, c)
            && self.empty([(i, j + 1), (i - 1, j + 1), (i - 1, j), (i, j - 1)])
        {
            self.upd(i, j);
        }
    }
}
