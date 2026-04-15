//! Attention Spotlight
//!
//! Spotlight model of attention.

/// Attention spotlight
pub struct Spotlight {
    position: (f32, f32),
    radius: f32,
    gradient: f32,
}

impl Spotlight {
    pub fn new(radius: f32) -> Self {
        Self {
            position: (0.0, 0.0),
            radius,
            gradient: 0.5,
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn intensity_at(&self, x: f32, y: f32) -> f32 {
        let dx = x - self.position.0;
        let dy = y - self.position.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > self.radius {
            0.0
        } else {
            1.0 - (dist / self.radius) * self.gradient
        }
    }
}
