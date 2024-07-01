use bevy::prelude::*;

use crate::{
    draw::{vertex::DrawKey, Drawer},
    util::{
        math::{equal, sin, sqrt, vec_angle, Interp::Linear, Interpolation},
        FloatExt, VecExt,
    },
};

impl<'a> Drawer<'a> {
    pub fn line(&mut self, state: LineState, layer: f32, x_from: f32, y_from: f32, x_to: f32, y_to: f32) {
        let LineState { key, stroke, colors } = state;

        let hs = stroke / 2.0;
        let mut dx = x_to - x_from;
        let mut dy = y_to - y_from;
        let len = sqrt(dx * dx + dy * dy);

        dx = dx / len * hs;
        dy = dy / len * hs;

        self.quad(
            key,
            layer,
            (x_from - dy, y_from + dx, colors[0]),
            (x_from + dy, y_from - dx, colors[1]),
            (x_to + dy, y_to - dx, colors[2]),
            (x_to - dy, y_to + dx, colors[3]),
        );
    }

    #[inline]
    pub fn line_angle(&mut self, state: LineState, layer: f32, x: f32, y: f32, angle: f32, len: f32) {
        let to = vec_angle(angle, len, 0.0);
        self.line(state, layer, x, y, x + to.x, y + to.y);
    }

    pub fn line_circle(&mut self, state: LineState, layer: f32, x: f32, y: f32, radius: f32, segments: usize) {
        let LineState { key, stroke, colors } = state;

        let mut lines = self.lines();
        for i in (0..segments).map(|i| i as f32) {
            let prog = i / segments as f32;
            let offset = vec_angle(prog * f32::PI2, radius, 0.0);

            lines.point(
                x + offset.x,
                y + offset.y,
                layer,
                Linear.interp(colors[0], colors[3], prog),
                Linear.interp(colors[1], colors[2], prog),
            );
        }

        lines.flush(key, stroke, true);
    }

    #[inline]
    pub fn lines<'t>(&'t mut self) -> Lines<'t, 'a> {
        Lines {
            drawer: self,
            points: Vec::new(),
        }
    }
}

pub struct Lines<'t, 'a> {
    drawer: &'t mut Drawer<'a>,
    points: Vec<(f32, f32, f32, Color, Color)>,
}

impl<'t, 'a> Lines<'t, 'a> {
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.points.reserve_exact(capacity);
    }

    #[inline]
    pub fn point(&mut self, x: f32, y: f32, layer: f32, left_color: Color, right_color: Color) {
        self.points.push((x, y, layer, left_color, right_color));
    }

    pub fn flush(self, key: DrawKey, stroke: f32, wrap: bool) {
        // Taken from https://github.com/earlygrey/shapedrawer/blob/master/drawer/src/space/earlygrey/shapedrawer/ShapeDrawer.java.
        let points = self.points;
        if points.len() < 2 {
            return
        }

        let drawer = self.drawer;
        let hw = stroke * 0.5;
        let len = points.len();

        drawer.requests.reserve(len);

        let straight = |a: Vec2, b: Vec2| {
            let ab = b - a;
            (Vec2::new(-ab.y, ab.x) + b, Vec2::new(ab.y, -ab.x) + b)
        };

        let join = |a: Vec2, b: Vec2, c: Vec2| {
            let ab = b - a;
            let bc = c - b;

            let angle = ab.angle_between(bc);
            if equal(angle, 0.0) {
                return straight(a, b)
            }

            let len = hw / sin(angle, 1.0, 1.0);
            let left = angle < 0.0;

            let ab = ab.set_length(len);
            let bc = bc.set_length(len);

            let [mut d, mut e] = [Vec2::ZERO; 2];
            let (inside, outside) = if left { (&mut d, &mut e) } else { (&mut e, &mut d) };
            *inside = b - ab + bc;
            *outside = b + ab - bc;

            (d, e)
        };

        let end = |a: Vec2, b: Vec2| {
            let v = (b - a).set_length(hw);
            (Vec2::new(v.y, -v.x) + b, Vec2::new(-v.y, v.x) + b)
        };

        let mut push = |layer: f32, p1: Vec2, c1: Color, p2: Vec2, c2: Color, p3: Vec2, c3: Color, p4: Vec2, c4: Color| {
            drawer.quad(
                key,
                layer,
                (p1.x, p1.y, c1),
                (p2.x, p2.y, c2),
                (p3.x, p3.y, c3),
                (p4.x, p4.y, c4),
            );
        };

        let [mut p1, mut p2] = [Vec2::ZERO; 2];
        for i in 1..(len - 1) {
            let a = Vec2::new(points[i - 1].0, points[i - 1].1);
            let b = Vec2::new(points[i].0, points[i].1);
            let c = Vec2::new(points[i + 1].0, points[i + 1].1);

            let (p3, p4) = join(a, b, c);
            if i == 1 {
                let (left, right) = if !wrap {
                    let (left, right) = end(b, a);
                    (right, left)
                } else {
                    join(Vec2::new(points[len - 1].0, points[len - 1].1), a, b)
                };

                p1 = right;
                p2 = left;
            }

            push(
                (points[i - 1].2 + points[i - 1].2) / 2.0,
                p1,
                points[i - 1].3,
                p2,
                points[i - 1].4,
                p3,
                points[i].4,
                p4,
                points[i].3,
            );

            p1 = p4;
            p2 = p3;

            if i == len - 2 {
                let (p3, p4) = if !wrap {
                    end(b, c)
                } else {
                    join(b, c, Vec2::new(points[0].0, points[0].1))
                };

                push(
                    (points[i].2 + points[i].2) / 2.0,
                    p1,
                    points[i].3,
                    p2,
                    points[i].4,
                    p3,
                    points[i + 1].4,
                    p4,
                    points[i + 1].3,
                );

                if wrap {
                    let (p1, p2) = join(
                        Vec2::new(points[len - 1].0, points[len - 1].1),
                        Vec2::new(points[0].0, points[0].1),
                        Vec2::new(points[1].0, points[1].1),
                    );

                    push(
                        (points[i + 1].2 + points[0].2) / 2.0,
                        p4,
                        points[i].3,
                        p3,
                        points[i].4,
                        p1,
                        points[i + 1].4,
                        p2,
                        points[i + 1].3,
                    )
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct LineState {
    pub key: DrawKey,
    pub stroke: f32,
    pub colors: [Color; 4],
}

impl LineState {
    #[inline]
    pub fn stroke(mut self, stroke: f32) -> Self {
        self.stroke = stroke;
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.colors = [color; 4];
        self
    }

    #[inline]
    pub fn color_tip(mut self, from: Color, to: Color) -> Self {
        self.colors = [from, from, to, to];
        self
    }

    #[inline]
    pub fn color_edge(mut self, left: Color, right: Color) -> Self {
        self.colors = [left, right, right, left];
        self
    }
}

impl Default for LineState {
    #[inline]
    fn default() -> Self {
        Self {
            key: default(),
            stroke: 1.0,
            colors: [Color::WHITE; 4],
        }
    }
}
