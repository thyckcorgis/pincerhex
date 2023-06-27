use std::iter;

use eframe::{egui, epaint};
use egui::{Color32, Pos2, Stroke};

use crate::app::Dimensions;

pub struct BoardCell {
    pub piece: Option<Piece>,
    pub idx: (i16, i16),
}

pub enum Piece {
    Black,
    White,
}

fn hex_points(center: Pos2, radius: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let angle = (std::f32::consts::PI / 3. * (i as f32)) + (std::f32::consts::PI / 6.);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            Pos2::new(x, y)
        })
        .collect()
}

pub fn hex_border(ui: &mut egui::Ui, dimensions: &Dimensions, center: Pos2, cell: BoardCell) {
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
                let (width, colour) = match (idx, cell.idx, dimensions.horizontal) {
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

pub fn hexagon(ui: &mut egui::Ui, size: f32, center: Pos2, cell: BoardCell) {
    let radius = size / 2.;
    let points = hex_points(center, radius);

    ui.painter().add(epaint::Mesh {
        indices: vec![0, 1, 2, 3, 4, 5, 0, 2, 3, 0, 5, 3],
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
}
