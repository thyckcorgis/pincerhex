use eframe::egui;
use egui::{Align, Layout, Pos2, Vec2};

use crate::board::{hex_border, hexagon, BoardCell, Piece};

const APP_KEY: &str = "pincerhex-app";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PincerhexApp {
    player_is_white: bool,
    new_game: bool,
    move_count: u16,
    // TODO: Use a better data structure
    board: Vec<BoardCell>,
    active: Piece,
}

impl Default for PincerhexApp {
    fn default() -> Self {
        let size = Dimensions::default().board_size;
        Self {
            player_is_white: true,
            new_game: true,
            move_count: 0,
            active: Piece::White,
            board: {
                let mut board = Vec::with_capacity((size * size) as usize);
                (0..size).for_each(|y| {
                    (0..size).for_each(|x| {
                        board.push(BoardCell {
                            piece: None,
                            idx: (x, y),
                        });
                    });
                });
                board
            },
        }
    }
}

impl eframe::App for PincerhexApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
            ui.label(match (self.move_count, self.player_is_white) {
                (0, true) => "Place a piece anywhere to start.",
                (0, false) => {
                    // TODO: Make a move here
                    ""
                }
                (_, _) => "Your turn.",
            });
        });
        let rect_size = ctx.screen_rect().size();
        let dimensions = Dimensions::from_rect(rect_size.x, rect_size.y);
        self.cells(&dimensions, ui);
    }

    fn cells(&mut self, dimensions: &Dimensions, ui: &mut egui::Ui) {
        let (next_y, next_x) = if dimensions.horizontal {
            (RIGHT_DOWN, RIGHT)
        } else {
            (LEFT_DOWN, RIGHT_DOWN)
        };
        let start = dimensions.start();
        let size = dimensions.hex_radius() * SQRT_3;
        let next_y = size * next_y;
        let next_x = size * next_x;
        self.board.iter_mut().for_each(|cell| {
            let (x, y) = cell.idx;
            let pos = start + next_y * y as f32 + next_x * x as f32;
            let res = hexagon(ui, dimensions.hex_size, pos, cell);
            if res.clicked() {
                if cell.piece.is_none() {
                    cell.piece = Some(self.active);
                    self.active = self.active.other();
                }
            }
        });

        self.board.iter().for_each(|&BoardCell { idx, .. }| {
            let (x, y) = idx;
            let pos = start + next_y * y as f32 + next_x * x as f32;
            hex_border(ui, dimensions, pos, idx);
        })
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
