use crate::{
    board::Board,
    tile::{Colour, PieceState, Tile},
};
use rand::Rng;

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

impl Edge {
    const fn idx(self) -> usize {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
            Self::Left => 2,
            Self::Right => 3,
        }
    }
    const fn colour(self) -> Colour {
        match self {
            Self::Top | Self::Bottom => Colour::Black,
            Self::Left | Self::Right => Colour::White,
        }
    }
}

pub struct PotEval<'a> {
    board: &'a Board,
    active: Colour,
    potential: Vec<[i32; 4]>,
    bridge: Vec<[f32; 4]>,
    update: Vec<bool>,
}

const INIT_POTENTIAL: i32 = 20000;
const DEFAULT_POTENTIAL: i32 = 128;
const DIFF: i32 = 140;
const MAX_VALUE: i32 = 30000;
const ROUNDS: i32 = 12;

const EDGES: [Edge; 4] = [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right];

impl<'a> PotEval<'a> {
    pub fn new(board: &'a Board, active: Colour) -> Self {
        let size = board.size;
        Self {
            board,
            active,
            potential: vec![[INIT_POTENTIAL; 4]; size.pow(2)],
            bridge: vec![[0.; 4]; size.pow(2)],
            update: vec![false; size.pow(2)],
        }
    }

    fn reset_update(&mut self) {
        for u in &mut self.update {
            *u = true;
        }
    }

    pub fn evaluate(&mut self) -> &mut Self {
        self.init_tile_potential();
        for i in EDGES {
            self.evaluate_side(i);
        }
        self
    }

    fn evaluate_side(&mut self, edge: Edge) {
        self.reset_update();
        for _ in 0..ROUNDS {
            let mut set = false;
            for (tile, state) in self.board.iter() {
                set |= self.set_pot(tile, state, edge);
            }
            for (tile, state) in self.board.iter().rev() {
                set |= self.set_pot(tile, state, edge);
            }
            if !set {
                break;
            }
        }
    }

    fn set_pot(&mut self, tile: Tile, state: PieceState, edge: Edge) -> bool {
        let index = tile.to_index(self.board.size).unwrap();
        self.update[index] = false;
        self.bridge[index][edge.idx()] = 0.;

        if matches!(state,PieceState::Colour(c) if c == edge.colour().opponent()) {
            return false;
        }

        // BEWARE: it all goes to shit from here

        let mut block_score = 0;
        let mut bridge_weights = [0; 6];
        let mut min_potential = MAX_VALUE;
        let mut neighbours = [0; 6];
        let mut total_weight = 0.;
        for (idx, value) in self
            .board
            .neighbours(tile)
            .iter()
            .map(|t| self.pot_val(*t, edge))
            .enumerate()
        {
            neighbours[idx] = value;
        }
        for idx in 0..6 {
            let value = neighbours[idx];
            if value >= MAX_VALUE && neighbours[(idx + 2) % 6] >= MAX_VALUE {
                if neighbours[(idx + 1) % 6] < 0 {
                    // view-source:https://www.lutanho.net/play/hex.html line 651:
                    // ddb=+32
                    // ^ original might have been a typo
                    block_score += 32;
                } else {
                    neighbours[(idx + 1) % 6] += 128; // 512
                }
            }
        }

        for idx in 0..6 {
            let value = neighbours[idx];
            if (value >= MAX_VALUE) && neighbours[(idx + 3) % 6] >= MAX_VALUE {
                block_score += 30;
            }
        }

        for idx in 0..6 {
            let value = neighbours[idx];
            if value < 0 {
                neighbours[idx] += MAX_VALUE;
                bridge_weights[idx] = 10;
            } else {
                bridge_weights[idx] = 1;
            }
            if min_potential > neighbours[idx] {
                min_potential = neighbours[idx];
            }
        }
        for idx in 0..6 {
            if neighbours[idx] == min_potential {
                total_weight += bridge_weights[idx] as f32;
            }
        }

        min_potential = self.score_bridges(edge, index, total_weight, &neighbours, min_potential);

        self.bridge[index][edge.idx()] += if self.is_inside_tile(tile) {
            block_score as f32
        } else {
            -2.
        };

        if self.is_corner_tile(tile) {
            self.bridge[index][edge.idx()] /= 2.;
        }

        self.bridge[index][edge.idx()] = f32::min(68., self.bridge[index][edge.idx()]);

        if matches!(state,PieceState::Colour(c) if c == edge.colour()) {
            if min_potential < self.potential[index][edge.idx()] {
                self.potential[index][edge.idx()] = min_potential;
                self.update_neighbours(tile);
                return true;
            }
            return false;
        } else if min_potential + DIFF < self.potential[index][edge.idx()] {
            self.potential[index][edge.idx()] = min_potential + DIFF;
            self.update_neighbours(tile);
            return true;
        }

        false
    }

    fn score_bridges(
        &mut self,
        edge: Edge,
        index: usize,
        total_weight: f32,
        neighbours: &[i32; 6],
        mut min_potential: i32,
    ) -> i32 {
        let edge_bridge_score = if edge.colour() == self.active {
            66.
        } else {
            52.
        };

        self.bridge[index][edge.idx()] = total_weight / 5.;
        if (2. ..10.).contains(&total_weight) {
            self.bridge[index][edge.idx()] = edge_bridge_score + total_weight - 2.;
            min_potential -= 32;
        }

        if total_weight < 2. {
            let mut closest_high_value = MAX_VALUE;
            for value in neighbours {
                if *value > min_potential && closest_high_value > *value {
                    closest_high_value = *value;
                }
            }

            if closest_high_value <= min_potential + 104 {
                self.bridge[index][edge.idx()] =
                    edge_bridge_score + (closest_high_value - min_potential) as f32 / 4.;
                min_potential -= 64;
            }
            min_potential += closest_high_value;
            min_potential /= 2;
        }

        min_potential
    }

    fn update_neighbours(&mut self, tile: Tile) {
        for n in self.board.neighbours(tile) {
            if let Some(idx) = n.and_then(|t| t.0.to_index(self.board.size)) {
                self.update[idx] = true;
            }
        }
    }

    fn pot_val(&self, tile: Option<(Tile, PieceState)>, edge: Edge) -> i32 {
        match tile {
            Some((_, PieceState::Colour(other))) if other == edge.colour().opponent() => MAX_VALUE, // Blocked
            None => MAX_VALUE, // Border
            Some((t, PieceState::Empty)) => self.get_potential(t, edge),
            Some((t, _)) => self.get_potential(t, edge) - MAX_VALUE,
        }
    }

    fn get_potential(&self, tile: Tile, edge: Edge) -> i32 {
        self.potential[tile.to_index(self.board.size).unwrap()][edge.idx()]
    }

    pub fn get_best_move(&self, move_count: u32) -> Tile {
        let mut ff: f32 = 0.0;
        let mut mm: f32 = 20000.0;
        let (iq, jq) = self.get_quadrant();
        let mut best_move: Option<Tile> = None;

        if move_count > 0 {
            ff = 190.0 / ((move_count * move_count) as f32);
        }

        let mut moves = std::collections::HashMap::new();
        for i in 0..self.board.size {
            for j in 0..self.board.size {
                if self.board.get(i as u8, j as u8) == Some(PieceState::Empty) {
                    continue;
                }

                let mut mmp = ((i as f32 - 5.0).abs() + (j as f32 - 5.0).abs()).mul_add(ff, rand::thread_rng().gen::<f32>());
                mmp += 8.0 * ((iq * (i as i32 - 5)) + (jq * (j as i32 - 5))) as f32
                    / (move_count + 1) as f32;

                let tile = Tile::Valid(i as u8, j as u8);
                let index = tile.to_index(self.board.size).unwrap();

                for val in &self.bridge[index] {
                    mmp -= *val;
                }

                let pp0 = self.potential[index][0] + self.potential[index][1];
                let pp1 = self.potential[index][2] + self.potential[index][3];
                mmp += (pp0 + pp1) as f32;

                if pp0 <= 268 || pp1 <= 268 {
                    mmp -= 400.0;
                }

                moves.insert(tile, mmp);

                if mmp < mm {
                    mm = mmp;
                    best_move = Some(tile);
                }
            }
        }

        best_move.unwrap()
    }

    fn get_quadrant(&self) -> (i32, i32) {
        let mut iq: i32 = 0;
        let mut jq: i32 = 0;
        let size = self.board.size as i32;
        for i in 0..size {
            for j in 0..size {
                if self.board.get(i as u8, j as u8) == Some(PieceState::Empty) {
                    iq += 2 * i + 1 - size;
                    jq += 2 * j + 1 - size;
                }
            }
        }
        (Self::sign(iq), Self::sign(jq))
    }

    const fn sign(x: i32) -> i32 {
        match x {
            0 => 0,
            i32::MIN..=-1 => -1,
            1..=i32::MAX => 1,
        }
    }

    fn init_tile_potential(&mut self) {
        let size = self.board.size;
        self.board.iter().for_each(|t| {
            let index = t.0.to_index(size).expect("valid index");
            for i in EDGES {
                match t.1 {
                    PieceState::Colour(c) if c == i.colour() => {
                        self.potential[index][i.idx()] = 0;
                    }
                    _ => {
                        self.potential[index][i.idx()] = DEFAULT_POTENTIAL;
                    }
                }
            }
        });
    }

    const fn is_corner_tile(&self, tile: Tile) -> bool {
        match tile {
            Tile::Valid(r, c)
                if ((r as usize) == 0 || (r as usize) == self.board.size - 1)
                    && ((c as usize) == 0 || (c as usize) == self.board.size - 1) =>
            {
                true
            }
            Tile::Valid(_, _) | Tile::Edge1 | Tile::Edge2 | Tile::Invalid => false,
        }
    }

    const fn is_inside_tile(&self, tile: Tile) -> bool {
        match tile {
            Tile::Valid(r, c)
                if (r as usize) < self.board.size - 1 && (c as usize) < self.board.size - 1 =>
            {
                true
            }
            Tile::Valid(_, _) | Tile::Edge1 | Tile::Edge2 | Tile::Invalid => false,
        }
    }
}
