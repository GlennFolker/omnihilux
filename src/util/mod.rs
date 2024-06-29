use std::{f32::consts::PI, mem};

use bevy::prelude::*;
use fastrand::Rng;
use float_next_after::NextAfter;

pub trait RngExt {
    fn range_f32(&mut self, from: f32, to: f32) -> f32;

    fn range_f64(&mut self, from: f64, to: f64) -> f64;
}

impl RngExt for Rng {
    #[inline]
    fn range_f32(&mut self, from: f32, to: f32) -> f32 {
        from + self.f32() * (to - from)
    }

    #[inline]
    fn range_f64(&mut self, from: f64, to: f64) -> f64 {
        from + self.f64() * (to - from)
    }
}

pub trait Vec3Ext {
    fn separate_z(self) -> (Vec2, f32);
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn separate_z(self) -> (Vec2, f32) {
        (Vec2 { x: self.x, y: self.y }, self.z)
    }
}

pub trait FloatExt {
    fn next_swap(&mut self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn next_swap(&mut self) -> Self {
        let next = self.next_after(Self::INFINITY);
        mem::replace(self, next)
    }
}

impl FloatExt for f64 {
    #[inline]
    fn next_swap(&mut self) -> Self {
        let next = self.next_after(Self::INFINITY);
        mem::replace(self, next)
    }
}

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
