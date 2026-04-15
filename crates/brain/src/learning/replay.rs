//! Experience Replay
//!
//! Storing and replaying experiences.

use std::collections::VecDeque;

/// Replay buffer
pub struct ReplayBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> ReplayBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, experience: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(experience);
    }

    pub fn sample(&self, batch_size: usize) -> Vec<T> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut samples: Vec<_> = self.buffer.iter().cloned().collect();
        samples.shuffle(&mut rng);
        samples.into_iter().take(batch_size).collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
