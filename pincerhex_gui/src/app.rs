use eframe::egui;
use egui::{Align, Layout, Pos2, Vec2};

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
        ui.with_layout(Layout::top_down(Align::Max), |ui| {
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
        let dimensions = Dimensions::from_rect(rect_size.x, rect_size.y);
        self.cells(&dimensions, ui);
        self.edges(&dimensions, ui);
    }

    fn cells(&mut self, dimensions: &Dimensions, ui: &mut egui::Ui) {
        let (next_y, next_x) = if dimensions.horizontal {
            (RIGHT_DOWN, RIGHT)
        } else {
            (LEFT_DOWN, RIGHT_DOWN)
        };
        let start = dimensions.start();
        (0..dimensions.board_size).for_each(|y| {
            let col_start = start + next_y * dimensions.hex_radius() * SQRT_3 * y as f32;
            (0..dimensions.board_size).for_each(|x| {
                let pos = col_start + next_x * dimensions.hex_radius() * SQRT_3 * x as f32;
                hexagon(
                    ui,
                    dimensions.hex_size,
                    pos,
                    BoardCell {
                        piece: None,
                        idx: (x, y),
                    },
                );
            })
        });
    }

    fn edges(&mut self, dimensions: &Dimensions, ui: &mut egui::Ui) {
        let (next_y, next_x) = if dimensions.horizontal {
            (RIGHT_DOWN, RIGHT)
        } else {
            (LEFT_DOWN, RIGHT_DOWN)
        };
        let start = dimensions.start();
        (0..dimensions.board_size).for_each(|y| {
            let col_start = start + next_y * dimensions.hex_radius() * SQRT_3 * y as f32;
            (0..dimensions.board_size).for_each(|x| {
                let pos = col_start + next_x * dimensions.hex_radius() * SQRT_3 * x as f32;
                hex_border(
                    ui,
                    dimensions,
                    pos,
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
pub struct Dimensions {
    pub hex_size: f32,
    pub board_size: i16,
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
    fn from_rect(x: f32, y: f32) -> Self {
        let mut dim = Self::default();
        dim.horizontal = x > y;
        // TODO: do some optimization with basic algebra
        dim.hex_size = (if dim.horizontal { 2.5 } else { 2. } * y) / (3. * dim.board_size as f32);
        dim.width = x;
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
const SQRT_3: f32 = 1.732_050_8;
const PI_3: f32 = std::f32::consts::PI / 3.;

const LEFT_DOWN: Vec2 = Vec2::new(-0.5, SQRT_3 / 2.);
const RIGHT_DOWN: Vec2 = Vec2::new(0.5, SQRT_3 / 2.);
const RIGHT: Vec2 = Vec2::new(1.0, 0.);
const LEFT: Vec2 = Vec2::new(-1.0, 0.);
// }}}

// vim:foldmethod=marker
