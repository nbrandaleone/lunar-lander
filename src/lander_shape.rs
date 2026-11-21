//! Vector wireframe geometry for the Apollo Lunar Module and crash debris.

use crate::ffi;
use crate::math::Vec2;

/// Number of independent wireframe parts that separate on crash.
pub const PART_COUNT: usize = 7;

/// Outward explosion velocities for each of the 7 lander parts.
pub const PART_VELOCITIES: [Vec2; PART_COUNT] = [
    Vec2::new(1.0, -2.5),  // Octagonal command module
    Vec2::new(2.0, -1.5),  // Base platform
    Vec2::new(0.0, -3.0),  // Left landing leg
    Vec2::new(3.0, -1.0),  // Right landing leg
    Vec2::new(1.0, -1.0),  // Left thruster nozzle
    Vec2::new(2.5, -1.0),  // Right thruster nozzle
    Vec2::new(2.0, -0.5),  // Bottom thruster rim
];

/// Emits canvas path commands for a single lander part (`0..7`).
pub unsafe fn draw_part(index: usize) {
    match index {
        0 => {
            // Octagonal command module
            let (min, max) = (2.6, 5.0);
            ffi::canvas_move_to(min, -max);
            ffi::canvas_line_to(max, -min);
            ffi::canvas_line_to(max, min);
            ffi::canvas_line_to(min, max);
            ffi::canvas_line_to(-min, max);
            ffi::canvas_line_to(-max, min);
            ffi::canvas_line_to(-max, -min);
            ffi::canvas_line_to(-min, -max);
            ffi::canvas_close_path();
        }
        1 => {
            // Base platform
            ffi::canvas_rect(-6.0, 5.0, 12.0, 2.0);
        }
        2 => {
            // Left leg and footpad
            ffi::canvas_move_to(-5.0, 7.5);
            ffi::canvas_line_to(-9.0, 13.0);
            ffi::canvas_move_to(-11.0, 13.0);
            ffi::canvas_line_to(-7.0, 13.0);
        }
        3 => {
            // Right leg and footpad
            ffi::canvas_move_to(5.0, 7.5);
            ffi::canvas_line_to(9.0, 13.0);
            ffi::canvas_move_to(11.0, 13.0);
            ffi::canvas_line_to(7.0, 13.0);
        }
        4 => {
            // Left thruster bell
            ffi::canvas_move_to(-3.0, 7.5);
            ffi::canvas_line_to(-5.0, 12.0);
            ffi::canvas_line_to(-4.5, 13.0);
        }
        5 => {
            // Right thruster bell
            ffi::canvas_move_to(3.0, 7.5);
            ffi::canvas_line_to(5.0, 12.0);
            ffi::canvas_line_to(4.5, 13.0);
        }
        _ => {
            // Bottom thruster rim
            ffi::canvas_move_to(4.0, 11.0);
            ffi::canvas_line_to(-4.0, 11.0);
        }
    }
}

/// Renders the 7-part lander wireframe and optional flickering exhaust plume.
pub fn render_lander(
    pos: Vec2,
    scale: f64,
    rotation_deg: f64,
    view_scale: f64,
    part_offsets: &[Vec2; PART_COUNT],
    thrust_build: f64,
    active: bool,
    counter: u64,
) {
    unsafe {
        ffi::canvas_save();
        ffi::canvas_translate(pos.x, pos.y);
        ffi::canvas_scale(scale, scale);
        ffi::canvas_rotate(rotation_deg * crate::math::TO_RADIANS);
        ffi::canvas_set_stroke(255, 255, 255, 1.0 / (scale * view_scale));
        ffi::canvas_begin_path();

        for (i, offset) in part_offsets.iter().enumerate() {
            ffi::canvas_save();
            ffi::canvas_translate(offset.x, offset.y);
            draw_part(i);
            ffi::canvas_restore();
        }

        if thrust_build > 0.01 && active {
            let flicker = ((((counter >> 1) % 3) as f64) * 0.2) + 1.0;
            let flame_len = 11.0 + thrust_build.min(1.0) * 20.0 * flicker;
            ffi::canvas_line_to(0.0, flame_len);
            ffi::canvas_close_path();
        }

        ffi::canvas_stroke();
        ffi::canvas_restore();
    }
}
