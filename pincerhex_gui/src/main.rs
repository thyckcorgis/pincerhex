use std::iter;

use eframe::{egui, epaint};
use egui::{Color32, Pos2, Stroke};

const HEX_SIZE: f32 = 50.;
const HEX_RADIUS: f32 = HEX_SIZE / 2.;
const BOARD_SIZE: i16 = 11;
const SQRT_3: f32 = 1.7320508075688772;
const PI_3: f32 = std::f32::consts::PI / 3.0;
const TOTAL_WIDTH: f32 = HEX_RADIUS * SQRT_3 * BOARD_SIZE as f32;
const TOTAL_HEIGHT: f32 = (HEX_SIZE * BOARD_SIZE as f32) + (HEX_RADIUS * (BOARD_SIZE - 1) as f32);

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        min_window_size: Some(egui::emath::Vec2 {
            x: TOTAL_WIDTH,
            y: TOTAL_HEIGHT + HEX_SIZE,
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Pincerhex",
        options,
        Box::new(|_cc| Box::<PincerhexApp>::default()),
    )
}

struct PincerhexApp {
    player_is_white: bool,
    new_game: bool,
}

impl Default for PincerhexApp {
    fn default() -> Self {
        Self {
            player_is_white: true,
            new_game: true,
        }
    }
}

fn right_down(p: Pos2, size: f32, mul: i16) -> Pos2 {
    let len = size * SQRT_3;
    let x = p.x + len * PI_3.cos() * mul as f32;
    let y = p.y + len * PI_3.sin() * mul as f32;
    Pos2::new(x, y)
}

fn left_down(p: Pos2, size: f32, mul: i16) -> Pos2 {
    let len = size * SQRT_3;
    let x = p.x - len * PI_3.cos() * mul as f32;
    let y = p.y + len * PI_3.sin() * mul as f32;
    Pos2::new(x, y)
}

fn hex_points(center: Pos2, radius: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let angle = (std::f32::consts::PI / 3.0 * (i as f32)) + (std::f32::consts::PI / 6.0);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            Pos2::new(x, y)
        })
        .collect()
}

enum Piece {
    Black,
    White,
}

struct BoardCell {
    piece: Option<Piece>,
    idx: (i16, i16),
}

fn hex_border(ui: &mut egui::Ui, size: f32, center: Pos2, cell: BoardCell) {
    let radius = size / 2.0;
    let points = hex_points(center, radius);

    points
        .iter()
        // Need another one to close the loop
        .chain(iter::once(points.first().unwrap()))
        .collect::<Vec<&Pos2>>()
        .windows(2)
        .enumerate()
        // TODO: Vary the stroke width depending on whether they're an edge cell
        .for_each(|(idx, pair)| {
            if let [a, b] = pair {
                let (width, colour) = match idx {
                    0 => (4., Color32::BLACK),
                    1 => (4., Color32::WHITE),
                    2 => (4., Color32::GREEN),
                    3 => (4., Color32::YELLOW),
                    4 => (4., Color32::BROWN),
                    5 => (4., Color32::BLUE),
                    _ => (1., Color32::TRANSPARENT),
                };
                ui.painter()
                    .line_segment([**a, **b], Stroke::new(width, colour));
            }
        });
}

fn hexagon(ui: &mut egui::Ui, size: f32, center: Pos2, cell: BoardCell) {
    let radius = size / 2.0;
    let points = hex_points(center, radius);

    ui.painter().add(epaint::Mesh {
        indices: vec![0, 1, 2, 3, 4, 5, 0, 2, 3, 0, 5, 3],
        vertices: points
            .iter()
            .map(|&pos| epaint::Vertex {
                pos,
                uv: Default::default(),
                color: Color32::from_rgb(255, 0, 0),
            })
            .collect(),
        ..Default::default()
    });

    points
        .iter()
        // Need another one to close the loop
        .chain(iter::once(points.first().unwrap()))
        .collect::<Vec<&Pos2>>()
        .windows(2)
        .for_each(|pair| {
            if let [a, b] = pair {
                ui.painter()
                    .line_segment([**a, **b], Stroke::new(1., Color32::BROWN));
            }
        });

    if let Some(colour) = cell.piece {
        ui.painter().circle_filled(
            center,
            radius / 1.5,
            match colour {
                Piece::Black => Color32::BLACK,
                Piece::White => Color32::WHITE,
            },
        );
    } else {
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}{}", (cell.idx.0 + 97) as u8 as char, cell.idx.1 + 1),
            epaint::FontId::default(),
            Color32::BLACK,
        );
    }
}

impl eframe::App for PincerhexApp {
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

    fn hex_board(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Pincerhex");
        });
        let start = Pos2::new(ctx.screen_rect().size().x / 2., HEX_SIZE as f32);
        (0..BOARD_SIZE).for_each(|y| {
            let col_start = left_down(start, HEX_RADIUS, y);
            (0..BOARD_SIZE).for_each(|x| {
                hexagon(
                    ui,
                    HEX_SIZE,
                    right_down(col_start, HEX_RADIUS, x),
                    BoardCell {
                        piece: None, // Some(Piece::Black),
                        idx: (x, y),
                    },
                );
            })
        });
        (0..BOARD_SIZE).for_each(|y| {
            let col_start = left_down(start, HEX_RADIUS, y);
            (0..BOARD_SIZE).for_each(|x| {
                hex_border(
                    ui,
                    HEX_SIZE,
                    right_down(col_start, HEX_RADIUS, x),
                    BoardCell {
                        piece: None, // Some(Piece::Black),
                        idx: (x, y),
                    },
                );
            })
        })
    }
}
