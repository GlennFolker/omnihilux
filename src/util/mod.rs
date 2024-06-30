use std::mem;

use bevy::prelude::*;
use fastrand::Rng;
use float_next_after::NextAfter;

pub mod math;

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

pub trait VecExt: Sized {
    #[inline]
    fn set_length(self, length: f32) -> Self {
        self.set_length_squared(length * length)
    }

    fn set_length_squared(self, length_squared: f32) -> Self;
}

impl VecExt for Vec2 {
    #[inline]
    fn set_length_squared(self, length_squared: f32) -> Self {
        let old = self.length_squared();
        if old == 0.0 || old == length_squared {
            self
        } else {
            self * fastapprox::faster::pow(length_squared / old, 0.5)
        }
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
    const PI: Self;
    const PI2: Self;

    fn next_swap(&mut self) -> Self;
}

impl FloatExt for f32 {
    const PI: Self = std::f32::consts::PI;
    const PI2: Self = Self::PI * 2.0;

    #[inline]
    fn next_swap(&mut self) -> Self {
        let next = self.next_after(Self::INFINITY);
        mem::replace(self, next)
    }
}

impl FloatExt for f64 {
    const PI: Self = std::f64::consts::PI;
    const PI2: Self = Self::PI * 2.0;

    #[inline]
    fn next_swap(&mut self) -> Self {
        let next = self.next_after(Self::INFINITY);
        mem::replace(self, next)
    }
}
