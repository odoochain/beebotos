//! Saliency Detection
//!
//! Detecting salient features in input.

/// Saliency map
pub struct SaliencyMap {
    values: Vec<f32>,
    width: usize,
    height: usize,
}

impl SaliencyMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            values: vec![0.0; width * height],
            width,
            height,
        }
    }

    pub fn compute(&mut self, input: &[f32]) {
        // Compute saliency
        for (i, v) in input.iter().enumerate().take(self.values.len()) {
            self.values[i] = v.abs();
        }
    }

    pub fn get_max_location(&self) -> (usize, usize) {
        if let Some(max_idx) = self.values.iter().enumerate().max_by(|a, b| crate::utils::compare_f32(a.1, b.1)) {
            let x = max_idx.0 % self.width;
            let y = max_idx.0 / self.width;
            (x, y)
        } else {
            (0, 0)
        }
    }
}
