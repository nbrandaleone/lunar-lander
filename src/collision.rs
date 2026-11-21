//! Terrain collision detection, landing validation, and altitude computation.

use crate::lander::Lander;
use crate::math::Vec2;
use crate::terrain::Landscape;

/// Outcome of checking the lander against the lunar surface for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionResult {
    /// Lander is airborne with no surface contact.
    None,
    /// Lander touched down safely on a flat pad with the given score multiplier.
    Landed { multiplier: u32, perfect: bool },
    /// Lander impacted terrain or missed/tilted on a pad.
    Crashed,
}

/// Checks whether a point lies on or below the sloped segment `(p1, p2)`.
pub fn point_below_line(point: Vec2, p1: Vec2, p2: Vec2) -> bool {
    let dx = p2.x - p1.x;
    if dx.abs() < f64::EPSILON {
        return false;
    }
    let t = (point.x - p1.x) / dx;
    let y_on_line = p1.y + (p2.y - p1.y) * t;
    t > 0.0 && t < 1.0 && y_on_line <= point.y
}

/// Evaluates lander-terrain collisions and updates `lander.altitude`.
pub fn check_collisions(lander: &mut Lander, landscape: &Landscape) -> CollisionResult {
    let tile_w = landscape.tile_width;
    let mut right = lander.right % tile_w;
    let mut left = lander.left % tile_w;

    while right < 0.0 {
        right += tile_w;
        left += tile_w;
    }

    for seg in &landscape.segments {
        if !(right < seg.p1.x || left > seg.p2.x) {
            lander.altitude = seg.p1.y - lander.bottom;

            if seg.landable {
                if lander.bottom >= seg.p1.y {
                    let within_pad = left > seg.p1.x && right < seg.p2.x;
                    let upright = lander.rotation.abs() < 0.01;
                    let slow_enough = lander.vel.y < 0.15;
                    if within_pad && upright && slow_enough {
                        return CollisionResult::Landed {
                            multiplier: seg.multiplier,
                            perfect: lander.vel.y < 0.075,
                        };
                    }
                    return CollisionResult::Crashed;
                }
            } else if lander.bottom > seg.p2.y || lander.bottom > seg.p1.y {
                let bl = Vec2::new(left, lander.bottom);
                let br = Vec2::new(right, lander.bottom);
                if point_below_line(bl, seg.p1, seg.p2) || point_below_line(br, seg.p1, seg.p2) {
                    return CollisionResult::Crashed;
                }
            }
        }
    }
    CollisionResult::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_below_sloped_segment_detects_impact() {
        let p1 = Vec2::new(0.0, 100.0);
        let p2 = Vec2::new(100.0, 200.0);
        assert!(point_below_line(Vec2::new(50.0, 160.0), p1, p2));
        assert!(!point_below_line(Vec2::new(50.0, 140.0), p1, p2));
    }

    #[test]
    fn landing_on_flat_pad_succeeds_when_upright_and_slow() {
        let landscape = Landscape::new(1);
        let pad = landscape.segments.iter().find(|s| s.landable && (s.p2.x - s.p1.x) > 30.0).unwrap();
        let mut lander = Lander::new();
        lander.scale = 0.25;
        lander.rotation = 0.0;
        lander.vel = Vec2::new(0.0, 0.05);
        lander.pos = Vec2::new((pad.p1.x + pad.p2.x) * 0.5, pad.p1.y - 14.0 * 0.25 + 0.1);
        lander.sync_bounds();

        let res = check_collisions(&mut lander, &landscape);
        assert_eq!(res, CollisionResult::Landed { multiplier: pad.multiplier, perfect: true });
    }
}
