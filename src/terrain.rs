//! Lunar landscape generation, horizontal wrapping, and CRT vector rendering.

use crate::camera::ViewPort;
use crate::ffi;
use crate::math::{Rng, Vec2};
use crate::terrain_data::{AVAILABLE_ZONES, LAND_SCALE, LAND_Y_OFFSET, TERRAIN_POINTS, ZONE_COMBIS};

/// A single line segment of the lunar surface.
#[derive(Clone, Copy, Debug)]
pub struct TerrainSegment {
    pub p1: Vec2,
    pub p2: Vec2,
    /// True when the segment is completely horizontal (`p1.y == p2.y`).
    pub landable: bool,
    /// Active score multiplier (`1` for normal pads, `2..=5` for bonus pads).
    pub multiplier: u32,
}

/// A background star positioned above the terrain.
#[derive(Clone, Copy, Debug)]
pub struct Star {
    pub x: f64,
    pub y: f64,
}

/// Infinite horizontally-tiling lunar landscape with stars and bonus pads.
pub struct Landscape {
    pub segments: Vec<TerrainSegment>,
    pub stars: Vec<Star>,
    pub tile_width: f64,
    current_combi: usize,
}

impl Landscape {
    /// Builds the lunar surface and procedural starfield from [`TERRAIN_POINTS`].
    pub fn new(seed: u64) -> Self {
        let mut scaled: Vec<Vec2> = TERRAIN_POINTS
            .iter()
            .map(|&(x, y)| Vec2::new(x * LAND_SCALE, y * LAND_SCALE + LAND_Y_OFFSET))
            .collect();
        let tile_width = TERRAIN_POINTS.last().map(|p| p.0 * LAND_SCALE).unwrap_or(895.275);

        let mut segments = Vec::with_capacity(scaled.len().saturating_sub(1));
        for pair in scaled.windows(2) {
            let (p1, p2) = (pair[0], pair[1]);
            segments.push(TerrainSegment {
                p1,
                p2,
                landable: (p1.y - p2.y).abs() < f64::EPSILON,
                multiplier: 1,
            });
        }
        scaled.clear();

        let mut rng = Rng::new(seed);
        let mut stars = Vec::new();
        for seg in &segments {
            if rng.next_f64() < 0.12 {
                let sy = rng.next_f64() * 600.0;
                if sy < seg.p1.y && sy < seg.p2.y {
                    stars.push(Star { x: seg.p1.x, y: sy });
                }
            }
        }
        if stars.is_empty() {
            stars.push(Star { x: 50.0, y: 80.0 });
        }

        let mut landscape = Self { segments, stars, tile_width, current_combi: 0 };
        landscape.cycle_zones();
        landscape
    }

    /// Activates the next combination of bonus landing pads.
    pub fn cycle_zones(&mut self) {
        for seg in &mut self.segments {
            seg.multiplier = 1;
        }
        let combi = ZONE_COMBIS[self.current_combi];
        for &zone_idx in &combi {
            let zone = AVAILABLE_ZONES[zone_idx];
            if let Some(seg) = self.segments.get_mut(zone.line_num) {
                seg.multiplier = zone.multiplier;
            }
        }
        self.current_combi = (self.current_combi + 1) % ZONE_COMBIS.len();
    }

    /// Renders the visible terrain, blinking pad multipliers, and starfield.
    pub fn render(&self, view: &ViewPort, counter: u64) {
        if self.segments.is_empty() {
            return;
        }
        let mut offset = 0.0;
        while view.left - offset > self.tile_width {
            offset += self.tile_width;
        }
        while view.left - offset < 0.0 {
            offset -= self.tile_width;
        }
        let start_offset = offset;
        let mut i = 0;
        while self.segments[i].p2.x + offset < view.left {
            i += 1;
            if i >= self.segments.len() {
                i = 0;
                offset += self.tile_width;
            }
        }

        let jitter_x = if counter % 3 == 0 { 0.2 / view.scale } else { 0.0 };
        let jitter_y = if counter % 3 == 0 { 0.2 / view.scale } else { 0.0 };
        let first = self.segments[i];

        unsafe {
            ffi::canvas_begin_path();
            ffi::canvas_move_to(first.p1.x + offset + jitter_x, first.p1.y + jitter_y);

            let mut seg_idx = i;
            let mut seg_off = offset;
            while self.segments[seg_idx].p1.x + seg_off < view.right {
                let seg = self.segments[seg_idx];
                ffi::canvas_line_to(seg.p2.x + seg_off, seg.p2.y);

                if (counter % 20) > 10 && seg.multiplier != 1 {
                    let mid_x = (seg.p1.x + seg.p2.x) * 0.5 + seg_off;
                    let sx = mid_x * view.scale + view.x;
                    let sy = (seg.p2.y + 4.0) * view.scale + view.y;
                    ffi::canvas_draw_zone_label(seg.multiplier, sx, sy);
                }

                seg_idx += 1;
                if seg_idx >= self.segments.len() {
                    seg_idx = 0;
                    seg_off += self.tile_width;
                }
            }

            let flicker = ((counter as f64) * 0.8).sin() * 0.5 + 0.5;
            if flicker > 0.5 {
                let ch = ((flicker - 0.5) * 100.0).round() as u8;
                ffi::canvas_set_stroke(ch, ch, ch, 2.0 / view.scale);
                ffi::canvas_stroke();
            }
            ffi::canvas_set_stroke(255, 255, 255, (1.0 / view.scale) * (flicker * 0.2 + 0.8));
            ffi::canvas_stroke();
        }
        self.render_stars(view, start_offset);
    }

    fn render_stars(&self, view: &ViewPort, start_offset: f64) {
        let mut i = 0;
        let mut offset = start_offset;
        while self.stars[i].x + offset < view.left {
            i += 1;
            if i >= self.stars.len() {
                i = 0;
                offset += self.tile_width;
            }
        }
        let px = 1.0 / view.scale;
        unsafe {
            ffi::canvas_begin_path();
            while self.stars[i].x + offset < view.right {
                let star = self.stars[i];
                let sx = star.x + offset;
                let mut sy = star.y;
                while view.bottom < sy {
                    sy -= 600.0;
                }
                ffi::canvas_rect(sx, sy, px, px);
                if sy - 600.0 > view.top {
                    ffi::canvas_rect(sx, sy - 600.0, px, px);
                }
                i += 1;
                if i >= self.stars.len() {
                    i = 0;
                    offset += self.tile_width;
                }
            }
            ffi::canvas_set_stroke(255, 255, 255, px);
            ffi::canvas_stroke();
        }
    }
}
