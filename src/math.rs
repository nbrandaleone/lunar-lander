//! 2D vector math utilities and a lightweight deterministic PRNG.

use std::f64::consts::PI;

/// Conversion factor from degrees to radians (`PI / 180`).
pub const TO_RADIANS: f64 = PI / 180.0;

/// Two-dimensional floating-point vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    /// Creates a new [`Vec2`] with the given coordinates.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Adds `other` to `self` in place.
    pub fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }

    /// Rotates the vector around the origin by `degrees` counter-clockwise.
    pub fn rotated(self, degrees: f64) -> Self {
        let rad = degrees * TO_RADIANS;
        let cos = rad.cos();
        let sin = rad.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}

/// Clamps `value` to the inclusive range `[min, max]`.
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Minimal Xorshift64 PRNG so starfields and crash messages need no external crate.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Initializes the generator with a non-zero seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    /// Returns a pseudo-random `f64` in `[0.0, 1.0)`.
    pub fn next_f64(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 11) as f64 / ((1u64 << 53) as f64)
    }

    /// Returns a pseudo-random `f64` in `[min, max)`.
    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_rotate_90_degrees() {
        let v = Vec2::new(0.0, -1.0).rotated(-90.0);
        assert!((v.x - (-1.0)).abs() < 1e-9);
        assert!(v.y.abs() < 1e-9);
    }

    #[test]
    fn clamp_bounds() {
        assert_eq!(clamp(-120.0, -90.0, 90.0), -90.0);
        assert_eq!(clamp(45.0, -90.0, 90.0), 45.0);
        assert_eq!(clamp(120.0, -90.0, 90.0), 90.0);
    }

    #[test]
    fn rng_range_stays_in_bounds() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let val = rng.range(200.0, 400.0);
            assert!((200.0..400.0).contains(&val));
        }
    }
}
