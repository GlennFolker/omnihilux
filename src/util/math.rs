use std::f32::consts::PI;

use bevy::prelude::*;

pub trait Interpolation<T, P> {
    fn interp(self, from: T, to: T, progress: P) -> T;
}

#[derive(Copy, Clone)]
pub enum Interp {
    Linear,
    PowIn(u32),
}

impl Interpolation<f32, f32> for Interp {
    #[inline]
    fn interp(self, from: f32, to: f32, progress: f32) -> f32 {
        let progress = match self {
            Self::Linear => progress,
            Self::PowIn(pow) => fastapprox::faster::pow(progress, pow as f32),
        };
        from + (to - from) * progress
    }
}

impl Interpolation<Color, f32> for Interp {
    #[inline]
    fn interp(self, from: Color, to: Color, progress: f32) -> Color {
        let [fr, fg, fb, fa] = from.as_linear_rgba_f32();
        let [tr, tg, tb, ta] = to.as_linear_rgba_f32();
        Color::RgbaLinear {
            red: self.interp(fr, tr, progress),
            green: self.interp(fg, tg, progress),
            blue: self.interp(fb, tb, progress),
            alpha: self.interp(fa, ta, progress),
        }
    }
}

#[inline]
pub fn curve(f: f32, from: f32, to: f32) -> f32 {
    if f < from {
        0.0
    } else if f > to {
        1.0
    } else {
        (f - from) / (to - from)
    }
}

#[inline]
pub fn equal(a: f32, b: f32) -> bool {
    equal_within(a, b, 0.0001)
}

#[inline]
pub fn equal_within(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() <= epsilon
}

#[inline]
pub fn sin(rad: f32, scl: f32, mag: f32) -> f32 {
    let rad = (rad * scl) % (PI * 2.0);
    fastapprox::faster::sinfull(rad) * mag
}

#[inline]
pub fn vec_angle(angle: f32, x: f32, y: f32) -> Vec2 {
    let angle = angle % (PI * 2.0);
    let (cos, sin) = (fastapprox::faster::cosfull(angle), fastapprox::faster::sinfull(angle));

    Vec2 {
        x: x * cos - y * sin,
        y: x * sin + y * cos,
    }
}
