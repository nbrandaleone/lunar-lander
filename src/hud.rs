//! HUD telemetry synchronization and low-fuel warning logic.

use crate::ffi;
use crate::lander::Lander;

/// High-level phase of the Lunar Lander arcade session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Waiting,
    Playing,
    Landed,
    Crashed,
    GameOver,
}

/// Pushes telemetry values and low-fuel warnings to the browser HUD.
pub fn sync_hud(state: GameState, lander: &Lander, score: u32, play_ticks: u64, counter: u64) {
    let time_ms = ((play_ticks as f64) * (1000.0 / 60.0)) as u32;
    let alt = lander.altitude.max(0.0).round() as i32;
    let vx = (lander.vel.x * 200.0).round() as i32;
    let vy = (lander.vel.y * 200.0).round() as i32;
    unsafe {
        ffi::hud_update(score, time_ms, lander.fuel.round() as i32, alt, vx, vy);
        if state == GameState::Playing && lander.fuel < 300.0 {
            if (counter % 50) < 30 {
                let code = if lander.fuel <= 0.0 {
                    ffi::OVERLAY_OUT_OF_FUEL
                } else {
                    ffi::OVERLAY_LOW_FUEL
                };
                ffi::hud_set_overlay(code, 0, 0);
                if counter % 50 == 0 {
                    ffi::audio_play_event(ffi::AUDIO_BEEP);
                }
            } else {
                ffi::hud_set_overlay(ffi::OVERLAY_NONE, 0, 0);
            }
        }
    }
}
