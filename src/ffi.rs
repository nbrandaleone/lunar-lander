//! Host FFI bindings for Canvas 2D rendering, HUD updates, and audio cues.
//!
//! When compiled for `wasm32`, these functions are imported from the browser
//! JavaScript runtime (`env`). For native `cargo test` builds, no-op stubs are
//! provided so game logic and physics can be unit-tested without a browser.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    pub fn canvas_clear(width: f64, height: f64);
    pub fn canvas_save();
    pub fn canvas_restore();
    pub fn canvas_translate(x: f64, y: f64);
    pub fn canvas_scale(sx: f64, sy: f64);
    pub fn canvas_rotate(radians: f64);
    pub fn canvas_set_stroke(r: u8, g: u8, b: u8, line_width: f64);
    pub fn canvas_begin_path();
    pub fn canvas_move_to(x: f64, y: f64);
    pub fn canvas_line_to(x: f64, y: f64);
    pub fn canvas_close_path();
    pub fn canvas_rect(x: f64, y: f64, w: f64, h: f64);
    pub fn canvas_stroke();
    pub fn canvas_draw_zone_label(text_val: u32, x: f64, y: f64);
    pub fn hud_update(score: u32, time_ms: u32, fuel: i32, alt: i32, vx: i32, vy: i32);
    pub fn hud_set_overlay(code: u32, points: u32, fuel_lost: u32);
    pub fn audio_set_thrust(level: f64);
    pub fn audio_play_event(event_id: u32);
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables)]
mod stubs {
    pub unsafe fn canvas_clear(width: f64, height: f64) {}
    pub unsafe fn canvas_save() {}
    pub unsafe fn canvas_restore() {}
    pub unsafe fn canvas_translate(x: f64, y: f64) {}
    pub unsafe fn canvas_scale(sx: f64, sy: f64) {}
    pub unsafe fn canvas_rotate(radians: f64) {}
    pub unsafe fn canvas_set_stroke(r: u8, g: u8, b: u8, line_width: f64) {}
    pub unsafe fn canvas_begin_path() {}
    pub unsafe fn canvas_move_to(x: f64, y: f64) {}
    pub unsafe fn canvas_line_to(x: f64, y: f64) {}
    pub unsafe fn canvas_close_path() {}
    pub unsafe fn canvas_rect(x: f64, y: f64, w: f64, h: f64) {}
    pub unsafe fn canvas_stroke() {}
    pub unsafe fn canvas_draw_zone_label(text_val: u32, x: f64, y: f64) {}
    pub unsafe fn hud_update(score: u32, time_ms: u32, fuel: i32, alt: i32, vx: i32, vy: i32) {}
    pub unsafe fn hud_set_overlay(code: u32, points: u32, fuel_lost: u32) {}
    pub unsafe fn audio_set_thrust(level: f64) {}
    pub unsafe fn audio_play_event(event_id: u32) {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use stubs::*;

/// Audio event identifiers passed to [`audio_play_event`].
pub const AUDIO_BEEP: u32 = 1;
pub const AUDIO_EXPLOSION: u32 = 2;
pub const AUDIO_LANDING: u32 = 3;

/// Overlay message identifiers passed to [`hud_set_overlay`].
pub const OVERLAY_NONE: u32 = 0;
pub const OVERLAY_START: u32 = 1;
pub const OVERLAY_PERFECT: u32 = 2;
pub const OVERLAY_HARD: u32 = 3;
pub const OVERLAY_CRASH_1: u32 = 4;
pub const OVERLAY_CRASH_2: u32 = 5;
pub const OVERLAY_CRASH_3: u32 = 6;
pub const OVERLAY_GAME_OVER: u32 = 7;
pub const OVERLAY_LOW_FUEL: u32 = 8;
pub const OVERLAY_OUT_OF_FUEL: u32 = 9;
