//! Dynamic camera viewport with automatic close-up landing zoom.

use crate::lander::Lander;

/// Tracks screen-space transform and visible world bounds.
#[derive(Clone, Debug)]
pub struct ViewPort {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub zoomed_in: bool,
    pub width: f64,
    pub height: f64,
}

impl ViewPort {
    /// Initializes a viewport for the given canvas dimensions.
    pub fn new(width: f64, height: f64) -> Self {
        let mut vp = Self {
            x: 0.0,
            y: 0.0,
            scale: height / 700.0,
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
            zoomed_in: false,
            width,
            height,
        };
        vp.recompute_bounds();
        vp
    }

    /// Updates canvas dimensions and preserves current zoom mode.
    pub fn resize(&mut self, width: f64, height: f64, lander: &mut Lander) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        self.set_zoom(self.zoomed_in, lander);
    }

    /// Switches between wide orbital view and 5x close-up landing view.
    pub fn set_zoom(&mut self, zoom: bool, lander: &mut Lander) {
        self.zoomed_in = zoom;
        if zoom {
            self.scale = (self.height / 700.0) * 5.0;
            self.x = -lander.pos.x * self.scale + (self.width * 0.5);
            self.y = -lander.pos.y * self.scale + (self.height * 0.25);
            lander.scale = 0.25;
        } else {
            self.scale = self.height / 700.0;
            self.x = 0.0;
            self.y = 0.0;
            lander.scale = 0.6;
        }
        lander.sync_bounds();
        self.recompute_bounds();
    }

    /// Pans the camera to keep the lander inside screen margins and auto-zooms near terrain.
    pub fn update(&mut self, lander: &mut Lander) {
        if !self.zoomed_in && lander.altitude < 70.0 {
            self.set_zoom(true, lander);
        } else if self.zoomed_in && lander.altitude > 160.0 {
            self.set_zoom(false, lander);
        }

        let margin_x = self.width * 0.2;
        let margin_top = self.height * 0.2;
        let margin_bottom = self.height * 0.3;

        let screen_x = lander.pos.x * self.scale + self.x;
        if screen_x < margin_x {
            self.x = -lander.pos.x * self.scale + margin_x;
        } else if screen_x > self.width - margin_x {
            self.x = -lander.pos.x * self.scale + self.width - margin_x;
        }

        let screen_y = lander.pos.y * self.scale + self.y;
        if screen_y < margin_top {
            self.y = -lander.pos.y * self.scale + margin_top;
        } else if screen_y > self.height - margin_bottom {
            self.y = -lander.pos.y * self.scale + self.height - margin_bottom;
        }

        self.recompute_bounds();
    }

    fn recompute_bounds(&mut self) {
        self.left = -self.x / self.scale;
        self.top = -self.y / self.scale;
        self.right = self.left + (self.width / self.scale);
        self.bottom = self.top + (self.height / self.scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_zooms_in_below_altitude_70() {
        let mut vp = ViewPort::new(1280.0, 700.0);
        let mut lander = Lander::new();
        lander.altitude = 50.0;
        vp.update(&mut lander);
        assert!(vp.zoomed_in);
        assert_eq!(lander.scale, 0.25);

        lander.altitude = 180.0;
        vp.update(&mut lander);
        assert!(!vp.zoomed_in);
        assert_eq!(lander.scale, 0.6);
    }
}
