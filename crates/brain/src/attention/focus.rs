//! Attention Focus
//!
//! Managing focused attention.

/// Focus window
pub struct FocusWindow {
    center: (f32, f32),
    size: f32,
    intensity: f32,
}

impl FocusWindow {
    pub fn new(center: (f32, f32), size: f32) -> Self {
        Self {
            center,
            size,
            intensity: 1.0,
        }
    }

    pub fn move_to(&mut self, target: (f32, f32), speed: f32) {
        self.center.0 += (target.0 - self.center.0) * speed;
        self.center.1 += (target.1 - self.center.1) * speed;
    }

    pub fn contains(&self, point: (f32, f32)) -> bool {
        let dx = point.0 - self.center.0;
        let dy = point.1 - self.center.1;
        (dx * dx + dy * dy).sqrt() < self.size
    }
}
