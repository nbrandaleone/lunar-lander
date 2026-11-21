//! Top-level game state machine, scoring, fuel management, and frame loop.

use crate::camera::ViewPort;
use crate::collision::{check_collisions, CollisionResult};
use crate::ffi;
use crate::hud::{sync_hud, GameState};
use crate::lander::Lander;
use crate::math::Rng;
use crate::terrain::Landscape;

/// Complete mutable state of the Lunar Lander game.
pub struct Game {
    pub state: GameState,
    pub lander: Lander,
    pub landscape: Landscape,
    pub view: ViewPort,
    pub score: u32,
    pub counter: u64,
    pub play_ticks: u64,
    pub restart_timer: u32,
    pub input_left: bool,
    pub input_right: bool,
    pub input_thrust: bool,
    rng: Rng,
}

impl Game {
    /// Creates a new game in the attract-mode `Waiting` state.
    pub fn new(width: f64, height: f64) -> Self {
        let mut game = Self {
            state: GameState::GameOver,
            lander: Lander::new(),
            landscape: Landscape::new(0x5EED_1979),
            view: ViewPort::new(width, height),
            score: 0,
            counter: 0,
            play_ticks: 0,
            restart_timer: 0,
            input_left: false,
            input_right: false,
            input_thrust: false,
            rng: Rng::new(0xA901_1011),
        };
        game.restart_level();
        game
    }

    /// Starts a fresh arcade credit with 1000 fuel and 0 score.
    pub fn start_new_game(&mut self) {
        self.lander.fuel = 1000.0;
        self.score = 0;
        self.counter = 0;
        self.play_ticks = 0;
        self.state = GameState::Playing;
        self.restart_level();
    }

    /// Resets the lander and cycles bonus landing zones for the next attempt.
    pub fn restart_level(&mut self) {
        self.lander.reset();
        self.landscape.cycle_zones();
        self.view.set_zoom(false, &mut self.lander);
        self.restart_timer = 0;

        if self.state == GameState::GameOver {
            self.state = GameState::Waiting;
            self.lander.vel.x = 2.0;
            unsafe { ffi::hud_set_overlay(ffi::OVERLAY_START, 0, 0); }
        } else {
            self.state = GameState::Playing;
            unsafe { ffi::hud_set_overlay(ffi::OVERLAY_NONE, 0, 0); }
        }
    }

    /// Advances the simulation by one fixed 60Hz frame and renders the scene.
    pub fn step(&mut self) {
        self.counter += 1;

        if self.state == GameState::Playing {
            self.play_ticks += 1;
            if self.input_left {
                self.lander.rotate(-1.0);
            } else if self.input_right {
                self.lander.rotate(1.0);
            }
            self.lander.set_thrust(if self.input_thrust { 1.0 } else { 0.0 });
        }

        self.lander.update();

        if self.state == GameState::Waiting && self.lander.altitude < 100.0 {
            self.state = GameState::GameOver;
            self.restart_level();
        }

        if matches!(self.state, GameState::Playing | GameState::Waiting) {
            match check_collisions(&mut self.lander, &self.landscape) {
                CollisionResult::Landed { multiplier, perfect } if self.state == GameState::Playing => {
                    self.handle_landed(multiplier, perfect);
                }
                CollisionResult::Crashed if self.state == GameState::Playing => {
                    self.handle_crashed();
                }
                CollisionResult::Landed { .. } | CollisionResult::Crashed => {
                    self.state = GameState::GameOver;
                    self.restart_level();
                }
                CollisionResult::None => {}
            }
        }

        if self.restart_timer > 0 {
            self.restart_timer -= 1;
            if self.restart_timer == 0 {
                self.restart_level();
            }
        }

        self.view.update(&mut self.lander);
        self.render();
        if self.counter % 4 == 0 {
            sync_hud(self.state, &self.lander, self.score, self.play_ticks, self.counter);
        }
    }

    /// Handles safe touchdown scoring and bonus fuel.
    pub fn handle_landed(&mut self, multiplier: u32, perfect: bool) {
        self.lander.land();
        let points = if perfect {
            self.lander.fuel += 50.0;
            50 * multiplier
        } else {
            self.lander.make_bounce();
            15 * multiplier
        };
        self.score += points;
        self.state = GameState::Landed;
        self.restart_timer = 240; // 4 seconds at 60Hz
        let code = if perfect { ffi::OVERLAY_PERFECT } else { ffi::OVERLAY_HARD };
        unsafe {
            ffi::hud_set_overlay(code, points, 0);
            ffi::audio_play_event(ffi::AUDIO_LANDING);
        }
    }

    /// Handles crash fuel loss, explosion debris, and game-over transition.
    pub fn handle_crashed(&mut self) {
        self.lander.crash();
        let fuel_lost = self.rng.range(200.0, 400.0).round();
        self.lander.fuel = (self.lander.fuel - fuel_lost).max(0.0);

        if self.lander.fuel < 1.0 {
            self.state = GameState::GameOver;
            unsafe { ffi::hud_set_overlay(ffi::OVERLAY_GAME_OVER, 0, fuel_lost as u32); }
        } else {
            self.state = GameState::Crashed;
            let r = self.rng.next_f64();
            let code = if r < 0.3 {
                ffi::OVERLAY_CRASH_1
            } else if r < 0.6 {
                ffi::OVERLAY_CRASH_2
            } else {
                ffi::OVERLAY_CRASH_3
            };
            unsafe { ffi::hud_set_overlay(code, 0, fuel_lost as u32); }
        }
        self.restart_timer = 240;
        unsafe { ffi::audio_play_event(ffi::AUDIO_EXPLOSION); }
    }

    fn render(&self) {
        unsafe {
            ffi::canvas_clear(self.view.width, self.view.height);
            ffi::canvas_save();
            ffi::canvas_translate(self.view.x, self.view.y);
            ffi::canvas_scale(self.view.scale, self.view.scale);
            self.landscape.render(&self.view, self.counter);
            self.lander.render(self.view.scale);
            ffi::canvas_restore();
        }
    }
}
