use std::iter;

use eframe::{egui, epaint};
use egui::{Color32, Pos2, Rect, Response, Stroke};

use crate::app::Dimensions;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BoardCell {
    pub piece: Option<Piece>,
    pub idx: (i16, i16),
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub enum Piece {
    Black,
    White,
}

impl Piece {
    pub fn other(&self) -> Self {
        match self {
            Self::White => Piece::Black,
            Self::Black => Piece::White,
        }
    }
}

fn hex_points(center: Pos2, radius: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            use std::f32::consts::PI;
            let angle = (PI / 3. * (i as f32)) + (PI / 6.);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            Pos2::new(x, y)
        })
        .collect()
}

pub fn hex_border(ui: &mut egui::Ui, dimensions: &Dimensions, center: Pos2, (x, y): (i16, i16)) {
    let radius = dimensions.hex_size / 2.;
    let points = hex_points(center, radius);

    // TODO: split the vertical segments in half to create the "split" border
    // Not sure if it's worth our time to do that right now though
    points
        .iter()
        // Need another one to close the loop
        .chain(iter::once(points.first().unwrap()))
        .collect::<Vec<&Pos2>>()
        .windows(2)
        .enumerate()
        .for_each(|(idx, pair)| {
            if let [a, b] = pair {
                let (width, colour) = match (idx, (x, y), dimensions.horizontal) {
                    (2 | 3, (0, _), false) | (1 | 2, (0, _), true) => (4., Color32::WHITE),
                    (4 | 5, (_, 0), false) | (3 | 4, (_, 0), true) => (4., Color32::BLACK),
                    (0 | 5, (x, _), false) | (4 | 5, (x, _), true)
                        if x == dimensions.board_size - 1 =>
                    {
                        (4., Color32::WHITE)
                    }
                    (1 | 2, (_, y), false) | (0 | 1, (_, y), true)
                        if y == dimensions.board_size - 1 =>
                    {
                        (4., Color32::BLACK)
                    }
                    _ => (0., Color32::TRANSPARENT),
                };
                ui.painter()
                    .line_segment([**a, **b], Stroke::new(width, colour));
            }
        });
}

const HEXAGON_INDICES: [u32; 12] = [0, 1, 2, 3, 4, 5, 0, 2, 3, 0, 5, 3];

pub fn hexagon(ui: &mut egui::Ui, size: f32, center: Pos2, cell: &BoardCell) -> Response {
    let radius = size / 2.;
    let points = hex_points(center, radius);

    ui.painter().add(epaint::Mesh {
        indices: HEXAGON_INDICES.to_vec(),
        vertices: points
            .iter()
            .map(|&pos| epaint::Vertex {
                pos,
                uv: Default::default(),
                color: Color32::KHAKI,
            })
            .collect(),
        ..Default::default()
    });

    // TODO: Fix this offset to only take half of the non-rectangular part of the hexagon.
    // Right now the lower cells get precedence because of the overlap
    let offset = egui::Vec2::new(radius, radius);
    let hitbox = Rect::from_two_pos(center - offset, center + offset);
    let response = ui.allocate_rect(hitbox, egui::Sense::click());

    if response.hovered() {
        ui.painter().add(epaint::Mesh {
            indices: HEXAGON_INDICES.to_vec(),
            vertices: points
                .iter()
                .map(|&pos| epaint::Vertex {
                    pos,
                    uv: Default::default(),
                    color: Color32::from_white_alpha(1),
                })
                .collect(),
            ..Default::default()
        });
    }

    points
        .iter()
        // Need another one to close the loop
        .chain(iter::once(points.first().unwrap()))
        .collect::<Vec<&Pos2>>()
        .windows(2)
        .for_each(|pair| {
            if let [a, b] = pair {
                ui.painter()
                    .line_segment([**a, **b], Stroke::new(1., Color32::DARK_GRAY));
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
    response
}
