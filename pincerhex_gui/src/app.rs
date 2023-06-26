use eframe::egui;
use egui::Pos2;

use crate::board::{hex_border, hexagon, BoardCell};

const APP_KEY: &str = "pincerhex-app";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PincerhexApp {
    player_is_white: bool,
    new_game: bool,
    move_count: u16,
}

impl Default for PincerhexApp {
    fn default() -> Self {
        Self {
            player_is_white: true,
            new_game: true,
            move_count: 0,
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
            }
        });
    }

    fn restart(&mut self) {
        let new_app = Self::default();
        self.new_game = new_app.new_game;
        self.player_is_white = new_app.player_is_white;
    }

    fn hex_board(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Pincerhex");
            if ui.button("New Game").clicked() {
                self.restart()
            }
            match (self.move_count, self.player_is_white) {
                (0, true) => {
                    ui.label("Place a piece anywhere to start.");
                }
                (0, false) => {
                    // TODO: Make a move here
                }
                (_, _) => {
                    ui.label("Your turn.");
                }
            }
        });
        let rect_size = ctx.screen_rect().size();
        let dimensions = Dimensions::from_height(rect_size.y);
        let padding = dimensions.hex_size * 0.75;
        let start = Pos2::new(rect_size.x / 2., padding);
        self.cells(start, &dimensions, ui);
        self.edges(start, &dimensions, ui);
    }

    fn cells(&mut self, start: Pos2, dimensions: &Dimensions, ui: &mut egui::Ui) {
        (0..dimensions.board_size).for_each(|y| {
            let col_start = start.left_down(dimensions.hex_radius(), y);
            (0..dimensions.board_size).for_each(|x| {
                hexagon(
                    ui,
                    dimensions.hex_size,
                    col_start.right_down(dimensions.hex_radius(), x),
                    BoardCell {
                        piece: None,
                        idx: (x, y),
                    },
                );
            })
        });
    }

    fn edges(&mut self, start: Pos2, dimensions: &Dimensions, ui: &mut egui::Ui) {
        (0..dimensions.board_size).for_each(|y| {
            let col_start = start.left_down(dimensions.hex_radius(), y);
            (0..dimensions.board_size).for_each(|x| {
                hex_border(
                    ui,
                    dimensions.hex_size,
                    col_start.right_down(dimensions.hex_radius(), x),
                    dimensions.board_size,
                    BoardCell {
                        piece: None,
                        idx: (x, y),
                    },
                );
            })
        })
    }
}

// {{{ Dimensions
#[derive(Debug)]
struct Dimensions {
    hex_size: f32,
    board_size: i16,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            hex_size: 40.,
            board_size: 11,
        }
    }
}

impl Dimensions {
    // TODO: Make board horizontal on desktop
    fn from_height(y: f32) -> Self {
        let mut dim = Self::default();
        dim.hex_size = (2. * y) / (3. * dim.board_size as f32);
        dim
    }

    fn hex_radius(&self) -> f32 {
        self.hex_size / 2.
    }
}
// }}}

// {{{ Hex
const SQRT_3: f32 = 1.732_050_8;
const PI_3: f32 = std::f32::consts::PI / 3.;

pub trait Hex {
    fn right_down(self, size: f32, mul: i16) -> Self;
    fn left_down(self, size: f32, mul: i16) -> Self;
}

impl Hex for Pos2 {
    fn right_down(self, size: f32, mul: i16) -> Self {
        let len = size * SQRT_3;
        let x = self.x + len * PI_3.cos() * mul as f32;
        let y = self.y + len * PI_3.sin() * mul as f32;
        Self::new(x, y)
    }

    fn left_down(self, size: f32, mul: i16) -> Self {
        let len = size * SQRT_3;
        let x = self.x - len * PI_3.cos() * mul as f32;
        let y = self.y + len * PI_3.sin() * mul as f32;
        Self::new(x, y)
    }
}
// }}}

// vim:foldmethod=marker
