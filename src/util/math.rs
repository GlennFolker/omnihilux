use bevy::prelude::*;

use crate::util::FloatExt;

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
            Self::PowIn(exp) => pow(progress, exp as f32),
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
    within(a, b, 0.0001)
}

#[inline]
pub fn within(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() <= epsilon
}

#[inline]
pub fn sin(rad: f32, scl: f32, mag: f32) -> f32 {
    fastapprox::faster::sin(mod_angle(rad * scl)) * mag
}

pub use fastapprox::faster::pow;

#[inline]
pub fn sqrt(f: f32) -> f32 {
    pow(f, 0.5)
}

#[inline]
pub fn mod_angle(angle: f32) -> f32 {
    let mut angle = angle % f32::PI2 + f32::PI2;
    if angle >= f32::PI2 {
        angle -= f32::PI2;
    }

    if angle >= f32::PI {
        angle -= f32::PI2;
    }

    angle
}

#[inline]
pub fn vec_angle(angle: f32, x: f32, y: f32) -> Vec2 {
    let angle = mod_angle(angle);
    let (cos, sin) = (fastapprox::faster::cos(angle), fastapprox::faster::sin(angle));

    Vec2 {
        x: x * cos - y * sin,
        y: x * sin + y * cos,
    }
}
