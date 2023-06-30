use rand::rngs::ThreadRng;
use rand::Rng;

#[cfg(debug_assertions)]
use crate::frame_history::FrameHistory;
use eframe::egui;
use egui::{Align, Layout, Pos2, Vec2};
use pincerhex_core::{first_move, PotentialEvaluator};

use crate::board::{hex_border, hexagon, Piece};
use pincerhex_state::{State, Winner};

const APP_KEY: &str = "pincerhex-app";

pub struct PincerhexState(State);

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializedState {
    active: Piece,
    pieces: Vec<(i8, i8, Piece)>,
    size: i8,
}

impl From<SerializedState> for PincerhexState {
    fn from(value: SerializedState) -> Self {
        use pincerhex_core::{PieceState, Tile};
        let mut state = State::new(value.size);
        state.set_to_play(value.active.into());
        for &(r, c, colour) in value.pieces.iter() {
            state
                .place_piece(Tile::Regular(r, c), PieceState::Colour(colour.into()))
                .unwrap();
        }
        Self(state)
    }
}

impl serde::Serialize for PincerhexState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use pincerhex_core::{PieceState, Tile};
        let board = self.0.get_board();
        let active: Piece = self.0.active().into();
        let mut pieces = Vec::new();
        for p in board.iter() {
            if let (Tile::Regular(r, c), PieceState::Colour(colour)) = p {
                pieces.push((r, c, colour.into()));
            }
        }

        SerializedState {
            active,
            pieces,
            size: board.size,
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PincerhexState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializedState::deserialize(deserializer).map(|x| x.into())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PincerhexApp {
    player_is_white: bool,
    new_game: bool,
    move_count: u16,
    state: PincerhexState,
    active: Piece,
    #[serde(skip)]
    won: Option<bool>,

    #[serde(skip)]
    #[cfg(debug_assertions)]
    frame_history: crate::frame_history::FrameHistory,
}

impl Default for PincerhexApp {
    fn default() -> Self {
        let size = Dimensions::default().board_size;
        let mut state = PincerhexState(State::new(size));
        state.0.set_to_play(Piece::White.into());
        Self {
            player_is_white: true,
            new_game: true,
            won: None,
            #[cfg(debug_assertions)]
            frame_history: FrameHistory::default(),
            move_count: 0,
            active: Piece::White,
            state,
        }
    }
}

impl eframe::App for PincerhexApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(_) = self.won {
            eframe::set_value(storage, APP_KEY, &PincerhexApp::default());
        } else {
            eframe::set_value(storage, APP_KEY, self);
        }
    }

    fn update(&mut self, ctx: &egui::Context, #[allow(dead_code)] frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(debug_assertions)]
            self.frame_history
                .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
            if self.new_game {
                self.start_menu(ui);
            } else {
                self.hex_board(ctx, ui);
            }
        });
    }
}

impl PincerhexApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn start_menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("New Game");
            let label = if self.player_is_white {
                "White"
            } else {
                "Black"
            };
            ui.add(egui::Checkbox::new(
                &mut self.player_is_white,
                format!("Player colour: {label}"),
            ));
            if ui.add(egui::Button::new("Start game")).clicked() {
                self.new_game = false;
                self.active = if self.player_is_white {
                    Piece::White
                } else {
                    use pincerhex_core::{PieceState, Tile};
                    let mut rand = StdRng(rand::thread_rng());
                    let (i, j) = first_move(self.state.0.get_board().size, &mut rand);
                    self.state
                        .0
                        .place_piece(Tile::Regular(i, j), PieceState::Colour(Piece::White.into()))
                        .expect("valid move");
                    self.move_count += 1;
                    Piece::Black
                };
            }
        });
    }

    fn restart(&mut self) {
        *self = Self::default();
    }

    fn hex_board(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(Layout::top_down(Align::Max), |ui| {
            ui.heading("Pincerhex");
            if ui.button("New Game").clicked() {
                self.restart()
            }
            ui.label(match (self.won, self.move_count, self.player_is_white) {
                (Some(true), _, _) => "You won!",
                (Some(false), _, _) => "You lost!",
                (None, 0, true) => "Place a piece anywhere to start.",
                (None, 0, false) => "",
                (None, _, _) => "Your turn.",
            });
        });
        let rect_size = ctx.screen_rect().size();
        let dimensions = Dimensions::from_rect(rect_size.x, rect_size.y);
        self.cells(&dimensions, ui);
        #[cfg(debug_assertions)]
        self.frame_history.ui(ui);
    }

    fn cells(&mut self, dimensions: &Dimensions, ui: &mut egui::Ui) {
        use pincerhex_core::{PieceState, Tile};
        let (next_y, next_x) = if dimensions.horizontal {
            (RIGHT_DOWN, RIGHT)
        } else {
            (LEFT_DOWN, RIGHT_DOWN)
        };
        let start = dimensions.start();
        let size = dimensions.hex_radius() * SQRT_3;
        let next_y = size * next_y;
        let next_x = size * next_x;
        if let Some((x, y)) = self
            .state
            .0
            .get_board()
            .iter()
            .find_map(|(tile, piece_state)| {
                if let Tile::Regular(x, y) = tile {
                    let piece = match piece_state {
                        PieceState::Colour(c) => Some(c.into()),
                        PieceState::Empty => None,
                    };
                    let pos = start + next_y * y as f32 + next_x * x as f32;
                    let ongoing = self.won.is_none();
                    let res = hexagon(ui, dimensions.hex_size, pos, piece, ongoing);
                    if res.clicked() && ongoing && piece.is_none() {
                        return Some((x, y));
                    }
                }
                None
            })
        {
            self.state
                .0
                .place_piece(Tile::Regular(x, y), PieceState::Colour(self.active.into()))
                .unwrap();
            self.move_count += 1;
            if let Some(Winner::Opponent) = self.state.0.get_winner(self.active.other().into()) {
                self.won = Some(true);
                return;
            }
            self.active = self.active.other();
            let mut rng = StdRng(rand::thread_rng());
            let (i, j) = PotentialEvaluator::new(
                self.state.0.get_board(),
                self.active.into(),
                self.active.into(),
            )
            .evaluate()
            .get_best_move(self.move_count, &mut rng);
            let mv = Tile::Regular(i, j);

            self.state
                .0
                .place_piece(mv, PieceState::Colour(self.active.into()))
                .expect("valid move");
            self.move_count += 1;

            if let Some(Winner::Bot) = self.state.0.get_winner(self.active.into()) {
                self.won = Some(false);
                return;
            }

            self.active = self.active.other();
        }

        self.state.0.get_board().iter().for_each(|(t, _)| {
            if let Tile::Regular(x, y) = t {
                let pos = start + next_y * y as f32 + next_x * x as f32;
                hex_border(ui, dimensions, pos, (x, y));
            }
        })
    }
}

struct StdRng(ThreadRng);

impl pincerhex_core::Rand for StdRng {
    fn in_range(&mut self, a: i8, b: i8) -> i8 {
        self.0.gen_range(a..b)
    }

    fn next(&mut self) -> f32 {
        self.0.gen::<f32>()
    }
}

// {{{ Dimensions
#[derive(Debug)]
pub struct Dimensions {
    pub hex_size: f32,
    pub board_size: i8,
    pub horizontal: bool,
    pub width: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            hex_size: 40.,
            board_size: 11,
            horizontal: false,
            width: 320.,
        }
    }
}

impl Dimensions {
    fn from_rect(w: f32, h: f32) -> Self {
        let mut dim = Self::default();
        let size = dim.board_size as f32;
        dim.horizontal = w > h;
        dim.hex_size = if dim.horizontal {
            f32::min(2. * h / (SQRT_3 * size), 2. * w / (2. + 3. * size))
        } else {
            f32::min(w / (SQRT_3 * size - 1.), 2. * h / (4. * size - 3.)) / (SQRT_3 / 2.)
        };
        dim.width = w;
        dim
    }

    fn hex_radius(&self) -> f32 {
        self.hex_size / 2.
    }

    fn padding(&self) -> f32 {
        self.hex_size * 0.75
    }

    fn start(&self) -> Pos2 {
        let padding = self.padding();
        Pos2::new(
            if self.horizontal {
                padding
            } else {
                self.width / 2.
            },
            padding,
        )
    }
}
// }}}

// {{{ Hex
pub const SQRT_3: f32 = 1.732_050_8;
const LEFT_DOWN: Vec2 = Vec2::new(-0.5, SQRT_3 / 2.);
const RIGHT: Vec2 = Vec2::new(1.0, 0.);
const RIGHT_DOWN: Vec2 = Vec2::new(0.5, SQRT_3 / 2.);
// }}}

// vim:foldmethod=marker
