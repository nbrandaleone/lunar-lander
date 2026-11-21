//! Lunar Module physics, rotation, fuel consumption, and vector rendering.

use crate::ffi;
use crate::lander_shape::{render_lander, PART_COUNT, PART_VELOCITIES};
use crate::math::{clamp, Vec2};

/// Lunar gravitational acceleration per 60Hz tick.
pub const GRAVITY: f64 = 0.0005;
/// Main engine acceleration per 60Hz tick at full throttle.
pub const THRUST_ACCEL: f64 = 0.0015;
/// Horizontal atmospheric/numerical drag factor.
pub const DRAG: f64 = 0.9997;

/// Represents the player-controlled Lunar Module.
pub struct Lander {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f64,
    pub target_rotation: f64,
    pub thrusting: f64,
    pub thrust_build: f64,
    pub fuel: f64,
    pub altitude: f64,
    pub scale: f64,
    pub active: bool,
    pub exploding: bool,
    pub bouncing: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub bottom_left: Vec2,
    pub bottom_right: Vec2,
    part_offsets: [Vec2; PART_COUNT],
    last_rotate_tick: u64,
    counter: u64,
}

impl Lander {
    /// Creates a new lander in its default starting state.
    pub fn new() -> Self {
        let mut lander = Self {
            pos: Vec2::new(110.0, 150.0),
            vel: Vec2::new(0.415, 0.0),
            rotation: -90.0,
            target_rotation: -90.0,
            thrusting: 0.0,
            thrust_build: 0.0,
            fuel: 1000.0,
            altitude: 0.0,
            scale: 0.6,
            active: true,
            exploding: false,
            bouncing: 0.0,
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
            bottom_left: Vec2::default(),
            bottom_right: Vec2::default(),
            part_offsets: [Vec2::default(); PART_COUNT],
            last_rotate_tick: 0,
            counter: 0,
        };
        lander.sync_bounds();
        lander
    }

    /// Resets position, velocity, and orientation for a new landing attempt.
    pub fn reset(&mut self) {
        self.pos = Vec2::new(110.0, 150.0);
        self.vel = Vec2::new(0.415, 0.0);
        self.rotation = -90.0;
        self.target_rotation = -90.0;
        self.thrusting = 0.0;
        self.thrust_build = 0.0;
        self.bouncing = 0.0;
        self.active = true;
        self.exploding = false;
        self.part_offsets = [Vec2::default(); PART_COUNT];
        self.sync_bounds();
    }

    /// Steps target rotation by `±15°` (clamped to `[-90°, 90°]`) with an 80ms cooldown.
    pub fn rotate(&mut self, direction: f64) {
        // At 60Hz, 5 ticks ≈ 83ms, matching the 80ms cooldown in Lander.js.
        if self.counter.saturating_sub(self.last_rotate_tick) >= 5 {
            self.target_rotation = clamp(self.target_rotation + direction * 15.0, -90.0, 90.0);
            self.last_rotate_tick = self.counter;
        }
    }

    /// Sets desired throttle power (`0.0..=1.0`).
    pub fn set_thrust(&mut self, power: f64) {
        self.thrusting = clamp(power, 0.0, 1.0);
    }

    /// Advances physics, fuel consumption, and debris/bounce animations by one 60Hz tick.
    pub fn update(&mut self) {
        self.counter += 1;
        self.rotation += (self.target_rotation - self.rotation) * 0.3;
        if (self.rotation - self.target_rotation).abs() < 0.1 {
            self.rotation = self.target_rotation;
        }

        if self.exploding {
            for i in 0..PART_COUNT {
                self.part_offsets[i].add_assign(PART_VELOCITIES[i]);
            }
        }

        if self.active {
            if self.fuel <= 0.0 {
                self.thrusting = 0.0;
            }
            self.thrust_build += (self.thrusting - self.thrust_build) * 0.2;
            if self.thrust_build > 0.001 {
                let accel = Vec2::new(0.0, -THRUST_ACCEL * self.thrust_build).rotated(self.rotation);
                self.vel.add_assign(accel);
                self.fuel -= 0.2 * self.thrust_build;
            } else {
                self.thrust_build = 0.0;
            }
            self.pos.add_assign(self.vel);
            self.vel.x *= DRAG;
            self.vel.y += GRAVITY;
            self.sync_bounds();
        } else if self.bouncing > 0.0 {
            self.pos.y += self.bouncing.sin() * 0.07;
            self.bouncing -= std::f64::consts::PI / 20.0;
        }

        if self.fuel < 0.0 {
            self.fuel = 0.0;
        }
        unsafe {
            ffi::audio_set_thrust(self.thrust_build.min(1.0));
        }
    }

    /// Marks the lander as landed and cuts engine thrust.
    pub fn land(&mut self) {
        self.active = false;
        self.thrust_build = 0.0;
        self.thrusting = 0.0;
    }

    /// Triggers the hard-landing bounce animation.
    pub fn make_bounce(&mut self) {
        self.bouncing = std::f64::consts::PI * 2.0;
    }

    /// Triggers crash state and separates the 7 vector parts.
    pub fn crash(&mut self) {
        self.rotation = 0.0;
        self.target_rotation = 0.0;
        self.active = false;
        self.exploding = true;
        self.thrust_build = 0.0;
        self.thrusting = 0.0;
    }

    /// Updates collision bounding box from current position and scale.
    pub fn sync_bounds(&mut self) {
        self.left = self.pos.x - 10.0 * self.scale;
        self.right = self.pos.x + 10.0 * self.scale;
        self.bottom = self.pos.y + 14.0 * self.scale;
        self.top = self.pos.y - 5.0 * self.scale;
        self.bottom_left = Vec2::new(self.left, self.bottom);
        self.bottom_right = Vec2::new(self.right, self.bottom);
    }

    /// Renders the lander (and flickering exhaust plume when thrusting).
    pub fn render(&self, view_scale: f64) {
        render_lander(
            self.pos,
            self.scale,
            self.rotation,
            view_scale,
            &self.part_offsets,
            self.thrust_build,
            self.active,
            self.counter,
        );
    }
}
