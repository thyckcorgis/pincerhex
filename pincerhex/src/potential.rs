use crate::{
    board::Board,
    tile::{Colour, PieceState, Tile},
};
use rand::Rng;

#[cfg(feature = "explore")]
use crate::explore::explore_other_moves;

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

struct Params {
    rounds: i32,

    init_potential: i32,
    default_potential: i32,
    diff: i32,
    max_value: i32,

    // Factor for FF
    bf: f32,
    wf: f32,

    pp_threshold: i32,
    mmp_deduction: f32,
    move_mul: f32,
}

#[cfg(feature = "params")]
const PARAMS: Params = Params {
    bf: 140.0,
    wf: 1.0,
    init_potential: 20_000,
    default_potential: 128,
    diff: 140,
    max_value: 30_000,
    rounds: 1000,
    pp_threshold: 268,
    move_mul: 8.0,
    mmp_deduction: 400.0,
};

#[cfg(not(feature = "params"))]
const PARAMS: Params = Params {
    bf: 140.0,
    wf: 1.0,
    init_potential: 20_000,
    default_potential: 128,
    diff: 140,
    max_value: 30_000,
    rounds: 1000,
    pp_threshold: 268,
    move_mul: 8.0,
    mmp_deduction: 400.0,
};

const EDGES: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];

impl<'a> PotEval<'a> {
    pub fn new(board: &'a Board, active: Colour) -> Self {
        let size = board.size as usize;
        Self {
            board,
            active,
            potential: vec![[PARAMS.init_potential; 4]; size.pow(2)],
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
        for _i in 1..PARAMS.rounds {
            let mut set = 0;
            for (tile, state) in self.board.iter() {
                if self.update[tile.to_index(self.board.size).unwrap()] {
                    set += self.set_pot(tile, state, edge);
                }
            }
            for (tile, state) in self.board.iter().rev() {
                if self.update[tile.to_index(self.board.size).unwrap()] {
                    set += self.set_pot(tile, state, edge);
                }
            }

            if set == 0 {
                break;
            }
        }
    }

    fn set_pot(&mut self, tile: Tile, state: PieceState, edge: Edge) -> i32 {
        let index = tile.to_index(self.board.size).unwrap();
        self.update[index] = false;
        self.bridge[index][edge.idx()] = 0.;

        // Blocked, can't update
        if matches!(state,PieceState::Colour(c) if c == edge.colour().opponent()) {
            return 0;
        }

        // BEWARE: it all goes to shit from here
        let (block_score, min_potential) = self.calculate_potential(tile, edge);

        if self.is_inside_tile(tile) {
            self.bridge[index][edge.idx()] += block_score as f32;
        } else {
            self.bridge[index][edge.idx()] -= 2.;
        }

        if self.is_corner_tile(tile) {
            self.bridge[index][edge.idx()] /= 2.;
        }

        self.bridge[index][edge.idx()] = f32::min(self.bridge[index][edge.idx()], 68.);

        match self.board.get_tile(tile) {
            Some(PieceState::Colour(c)) if c == edge.colour() => {
                if min_potential < self.potential[index][edge.idx()] {
                    self.potential[index][edge.idx()] = min_potential;
                    self.update_neighbours(tile);
                    1
                } else {
                    0
                }
            }
            Some(_) | None if min_potential + PARAMS.diff < self.potential[index][edge.idx()] => {
                self.potential[index][edge.idx()] = min_potential + PARAMS.diff;
                self.update_neighbours(tile);
                1
            }
            Some(_) | None => 0,
        }
    }

    fn calculate_potential(&mut self, tile: Tile, edge: Edge) -> (i32, i32) {
        let mut block_score = 0;
        let mut bridge_weights = [0; 6];
        let mut min_potential = PARAMS.max_value;
        let mut neighbours = [0; 6];

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
            if value >= PARAMS.max_value && neighbours[(idx + 2) % 6] >= PARAMS.max_value {
                if neighbours[(idx + 1) % 6] < 0 {
                    block_score += 32;
                } else {
                    neighbours[(idx + 1) % 6] += 128; // 512
                }
            }
        }

        for idx in 0..6 {
            let value = neighbours[idx];
            if (value >= PARAMS.max_value) && neighbours[(idx + 3) % 6] >= PARAMS.max_value {
                block_score += 30;
            }
        }

        for idx in 0..6 {
            let value = neighbours[idx];
            if value < 0 {
                neighbours[idx] += PARAMS.max_value;
                bridge_weights[idx] = 10;
            } else {
                bridge_weights[idx] = 1;
            }
            if min_potential > neighbours[idx] {
                min_potential = neighbours[idx];
            }
        }

        let min_potential = self.score_bridge(
            edge,
            tile.to_index(self.board.size).unwrap(),
            &bridge_weights,
            &neighbours,
            min_potential,
        );

        (block_score, min_potential)
    }

    fn score_bridge(
        &mut self,
        edge: Edge,
        index: usize,
        bridge_weights: &[i32; 6],
        neighbours: &[i32; 6],
        mut min_potential: i32,
    ) -> i32 {
        let mut total_weight = 0.;
        for idx in 0..6 {
            if neighbours[idx] == min_potential {
                total_weight += bridge_weights[idx] as f32;
            }
        }

        let edge_bridge_score = if edge.colour() == self.active {
            66.
        } else {
            52.
        };
        let mut bridge_score = total_weight / 5.;
        if (2. ..10.).contains(&total_weight) {
            bridge_score = edge_bridge_score + total_weight - 2.;
            min_potential -= 32;
        }

        if total_weight < 2. {
            let mut closest_high_value = PARAMS.max_value;
            for idx in 0..6 {
                let val = neighbours[idx];
                if val > min_potential && closest_high_value > val {
                    closest_high_value = val;
                }
            }

            if closest_high_value <= min_potential + 104 {
                bridge_score = edge_bridge_score - (closest_high_value - min_potential) as f32 / 4.;
                min_potential -= 64;
            }
            min_potential += closest_high_value;
            min_potential /= 2;
        }

        self.bridge[index][edge.idx()] = bridge_score;
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
            Some((_, PieceState::Colour(other))) if other == edge.colour().opponent() => {
                PARAMS.max_value
            } // Blocked
            Some((Tile::Valid(r, c), PieceState::Empty)) => self.get_potential(r, c, edge),
            Some((Tile::Valid(r, c), _)) => self.get_potential(r, c, edge) - PARAMS.max_value,
            Some((_, _)) | None => PARAMS.max_value, // Border
        }
    }

    fn get_potential(&self, r: i8, c: i8, edge: Edge) -> i32 {
        let idx = Tile::Valid(r, c)
            .to_index(self.board.size)
            .map_or_else(|| panic!("wtf {r} {c}"), |i| i);
        self.potential[idx][edge.idx()]
    }

    pub fn get_best_move(&self, move_count: u16) -> Tile {
        let mut ff: f32 = 0.0;
        let mut mm: f32 = f32::MAX;
        let (iq, jq) = self.get_quadrant();
        let mut best_move: Option<Tile> = None;

        if move_count > 0 {
            let colour = unsafe { crate::STARTING_COLOUR };
            // let colour = self.active;
            let factor = if colour == Colour::Black {
                PARAMS.bf
            } else {
                PARAMS.wf
            };
            let m = move_count as usize;
            ff = factor / (m * m) as f32;
        }

        let mut moves = std::collections::HashMap::new();
        for i in 0..self.board.size {
            for j in 0..self.board.size {
                if self.board.get(i, j) != Some(PieceState::Empty) {
                    continue;
                }

                let f_size = f32::from(self.board.size / 2);
                let mut mmp = ((f32::from(i) - f_size).abs() + (f32::from(j) - f_size).abs())
                    .mul_add(ff, rand::thread_rng().gen::<f32>());
                mmp += PARAMS.move_mul
                    * f32::from(
                        (iq * (i - self.board.size / 2)) + (jq * (j - self.board.size / 2)),
                    )
                    / (move_count + 1) as f32;

                let tile = Tile::Valid(i, j);
                let index = tile.to_index(self.board.size).unwrap();

                for val in &self.bridge[index] {
                    mmp -= *val;
                }

                let pp0 = self.potential[index][0] + self.potential[index][1];
                let pp1 = self.potential[index][2] + self.potential[index][3];
                mmp += (pp0 + pp1) as f32;

                if pp0 <= PARAMS.pp_threshold || pp1 <= PARAMS.pp_threshold {
                    mmp -= PARAMS.mmp_deduction;
                }

                moves.insert(tile, mmp);

                if mmp < mm {
                    mm = mmp;
                    best_move = Some(tile);
                }
            }
        }

        #[cfg(feature = "explore")]
        return explore_other_moves(self.board, moves, best_move.expect("finding the best move"));
        #[cfg(not(feature = "explore"))]
        return best_move.expect("finding the best move");
    }

    fn get_quadrant(&self) -> (i8, i8) {
        let mut iq: i32 = 0;
        let mut jq: i32 = 0;
        let size = self.board.size;
        for i in 0..size {
            for j in 0..size {
                if self.board.get(i, j) != Some(PieceState::Empty) {
                    iq += 2 * i32::from(i) + 1 - i32::from(size);
                    jq += 2 * i32::from(j) + 1 - i32::from(size);
                }
            }
        }
        (iq.signum() as i8, jq.signum() as i8)
    }

    const fn get_edges(&self, i: i8) -> [(Edge, Tile); 4] {
        [
            (Edge::Top, Tile::Valid(0, i)),
            (Edge::Bottom, Tile::Valid(self.board.size - 1, i)),
            (Edge::Left, Tile::Valid(i, 0)),
            (Edge::Right, Tile::Valid(i, self.board.size - 1)),
        ]
    }

    fn init_tile_potential(&mut self) {
        let size = self.board.size;
        for i in 0..size {
            for (e, j) in self.get_edges(i) {
                let index = j.to_index(self.board.size).unwrap();
                match self.board.get_tile(j) {
                    Some(PieceState::Colour(c)) if c == e.colour() => {
                        self.potential[index][e.idx()] = 0;
                    }
                    _ => {
                        self.potential[index][e.idx()] = PARAMS.default_potential;
                    }
                }
            }
        }
    }

    const fn is_corner_tile(&self, tile: Tile) -> bool {
        match tile {
            Tile::Valid(r, c)
                if (r == 0 || r == self.board.size - 1) && (c == 0 || c == self.board.size - 1) =>
            {
                true
            }
            Tile::Valid(_, _) | Tile::Edge1 | Tile::Edge2 | Tile::Invalid => false,
        }
    }

    const fn is_inside_tile(&self, tile: Tile) -> bool {
        match tile {
            Tile::Valid(r, c)
                if r > 0 && r < self.board.size - 1 && c > 0 && c < self.board.size - 1 =>
            {
                true
            }
            Tile::Valid(_, _) | Tile::Edge1 | Tile::Edge2 | Tile::Invalid => false,
        }
    }
}
