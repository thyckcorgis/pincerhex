use alloc::fmt::format;

use eframe::{egui, App};
use egui::{Align, Layout};

use pincerhex_core::{first_move, PotentialEvaluator, Rand};
use pincerhex_state::{State, Winner};

use crate::board::{hex_border, hexagon, Piece};
#[cfg(debug_assertions)]
use crate::frame_history::FrameHistory;
use crate::{
    dimensions::{Dimensions, LEFT_DOWN, RIGHT, RIGHT_DOWN, SQRT_3},
    rng::Rng,
    state::PincerhexState,
};

const APP_KEY: &str = "pincerhex-app";

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

    #[serde(skip)]
    rng: Rng,
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
            rng: Rng::default(),
        }
    }
}

impl eframe::App for PincerhexApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.won.is_some() {
            eframe::set_value(storage, APP_KEY, &PincerhexApp::default());
        } else {
            eframe::set_value(storage, APP_KEY, self);
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.rng.next();
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(debug_assertions)]
            self.frame_history
                .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
            if self.new_game {
                self.start_menu(ui);
            } else {
                self.hex_board(ctx, ui, frame);
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

    fn save_state(&mut self, frame: Option<&mut eframe::Frame>) {
        if let Some(storage) = frame.and_then(|f| f.storage_mut()) {
            self.save(storage);
        }
    }

    fn place_piece(
        &mut self,
        (i, j): (i8, i8),
        piece: Piece,
        frame: Option<&mut eframe::Frame>,
    ) -> bool {
        use pincerhex_core::{PieceState, Tile};
        self.state
            .0
            .place_piece(Tile::Regular(i, j), PieceState::Colour(piece.into()))
            .expect("valid move");
        self.move_count += 1;

        let bot_color = if self.player_is_white {
            Piece::Black
        } else {
            Piece::White
        };

        if let Some(winner) = self.state.0.get_winner(bot_color.into()) {
            self.won = Some(match winner {
                Winner::Opponent => true,
                Winner::Bot => false,
            });
            self.save_state(frame);
            true
        } else {
            self.active = self.active.other();
            #[cfg(target_arch = "wasm32")]
            {
                self.save_state(frame);
            }
            false
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
                format(format_args!("Player colour: {label}")),
            ));
            if ui.add(egui::Button::new("Start game")).clicked() {
                self.new_game = false;
                self.active = if self.player_is_white {
                    Piece::White
                } else {
                    let mv = first_move(self.state.0.get_board().size, &mut self.rng);
                    self.place_piece(mv, Piece::White, None);
                    Piece::Black
                };
            }
        });
    }

    fn restart(&mut self) {
        *self = Self::default();
    }

    fn hex_board(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
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
            #[cfg(debug_assertions)]
            {
                self.frame_history.ui(ui);
                ui.label(format(format_args!("{}", self.frame_history.fps())));
            }
        });
        let rect_size = ctx.screen_rect().size();
        let dimensions = Dimensions::new(rect_size.x, rect_size.y, self.state.0.get_board().size);
        self.cells(&dimensions, ui, frame);
    }

    fn cells(&mut self, dimensions: &Dimensions, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
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
        let mut clicked = None;
        for (tile, piece_state) in self.state.0.get_board().iter() {
            if let Tile::Regular(x, y) = tile {
                let piece = match piece_state {
                    PieceState::Colour(c) => Some(c.into()),
                    PieceState::Empty => None,
                };
                let pos = start + next_y * y as f32 + next_x * x as f32;
                let ongoing = self.won.is_none();
                let res = hexagon(ui, dimensions.hex_size, pos, piece, ongoing);
                if res.clicked() && ongoing && piece.is_none() && clicked.is_none() {
                    clicked = Some((x, y));
                }
            }
        }

        self.state.0.get_board().iter().for_each(|(t, _)| {
            if let Tile::Regular(x, y) = t {
                let pos = start + next_y * y as f32 + next_x * x as f32;
                hex_border(ui, dimensions, pos, (x, y));
            }
        });

        if let Some(mv) = clicked {
            if self.place_piece(mv, self.active, Some(frame)) {
                return;
            }
            let mv = PotentialEvaluator::new(
                self.state.0.get_board(),
                self.active.into(),
                self.active.into(),
            )
            .evaluate()
            .get_best_move(self.move_count, &mut self.rng);
            self.place_piece(mv, self.active, Some(frame));
        }
    }
}
