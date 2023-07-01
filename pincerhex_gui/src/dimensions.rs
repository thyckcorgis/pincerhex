use egui::{Pos2, Vec2};

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
    pub fn new(w: f32, h: f32, board_size: i8) -> Self {
        let mut dim = Self {
            width: w,
            board_size,
            horizontal: w > h,
            ..Default::default()
        };
        let size = dim.board_size as f32;
        dim.hex_size = if dim.horizontal {
            f32::min(2. * h / (SQRT_3 * size), 2. * w / (2. + 3. * size))
        } else {
            f32::min(w / (SQRT_3 * size - 1.), 2. * h / (4. * size - 3.)) / (SQRT_3 / 2.)
        };
        dim
    }

    pub fn hex_radius(&self) -> f32 {
        self.hex_size / 2.
    }

    fn padding(&self) -> f32 {
        self.hex_size * 0.75
    }

    pub fn start(&self) -> Pos2 {
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
pub const LEFT_DOWN: Vec2 = Vec2::new(-0.5, SQRT_3 / 2.);
pub const RIGHT: Vec2 = Vec2::new(1.0, 0.);
pub const RIGHT_DOWN: Vec2 = Vec2::new(0.5, SQRT_3 / 2.);
// }}}

// vim:foldmethod=marker
