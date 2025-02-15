use bevy::prelude::*;

use crate::{
    draw::{
        vertex::{DrawKey, DrawVertex},
        Drawer,
    },
    shape::vertex::Request,
    util::math::{sqrt, vec_angle},
};

impl<'a> Drawer<'a> {
    pub fn quad(
        &mut self,
        key: DrawKey,
        layer: f32,
        (x1, y1, col1): (f32, f32, Color),
        (x2, y2, col2): (f32, f32, Color),
        (x3, y3, col3): (f32, f32, Color),
        (x4, y4, col4): (f32, f32, Color),
    ) {
        self.requests.push(Request {
            layer,
            vertices: vec![
                DrawVertex::new(x1, y1, col1),
                DrawVertex::new(x2, y2, col2),
                DrawVertex::new(x3, y3, col3),
                DrawVertex::new(x4, y4, col4),
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
            key,
        });
    }

    pub fn tri(
        &mut self,
        key: DrawKey,
        layer: f32,
        (x1, y1, col1): (f32, f32, Color),
        (x2, y2, col2): (f32, f32, Color),
        (x3, y3, col3): (f32, f32, Color),
    ) {
        self.requests.push(Request {
            layer,
            vertices: vec![
                DrawVertex::new(x1, y1, col1),
                DrawVertex::new(x2, y2, col2),
                DrawVertex::new(x3, y3, col3),
            ],
            indices: vec![0, 1, 2],
            key,
        });
    }

    pub fn tri_angle(&mut self, state: TriState, layer: f32, x: f32, y: f32, angle: f32) {
        let TriState {
            key,
            width,
            length,
            colors,
        } = state;

        let Vec2 { x: mut x2, y: mut y2 } = vec_angle(angle, length, 0.0);
        x2 += x;
        y2 += y;

        let hw = width / 2.0;
        let mut dx = x2 - x;
        let mut dy = y2 - y;
        let len = sqrt(dx * dx + dy * dy);

        dx = dx / len * hw;
        dy = dy / len * hw;

        self.tri(
            key,
            layer,
            (x - dy, y + dx, colors[0]),
            (x + dy, y - dx, colors[0]),
            (x2, y2, colors[1]),
        );
    }
}

#[derive(Copy, Clone)]
pub struct TriState {
    pub key: DrawKey,
    pub width: f32,
    pub length: f32,
    pub colors: [Color; 2],
}

impl TriState {
    #[inline]
    pub fn size(mut self, width: f32, length: f32) -> Self {
        self.width = width;
        self.length = length;
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.colors = [color; 2];
        self
    }

    #[inline]
    pub fn color_tip(mut self, from: Color, to: Color) -> Self {
        self.colors = [from, to];
        self
    }
}

impl Default for TriState {
    #[inline]
    fn default() -> Self {
        Self {
            key: default(),
            width: 1.0,
            length: 1.0,
            colors: [Color::WHITE; 2],
        }
    }
}
