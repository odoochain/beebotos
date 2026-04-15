//! Kernel utilities

use std::time::{Duration, Instant};

/// Measure execution time
pub struct Timer {
    /// Start timestamp
    start: Instant,
    /// Timer name
    name: String,
}

impl Timer {
    /// Create new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    /// Elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Print elapsed time
    pub fn print(&self) {
        log::debug!("{} took {:?}", self.name, self.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.print();
    }
}

/// Rate limiter
pub struct RateLimiter {
    /// Last call timestamp
    last_call: Instant,
    /// Minimum interval between calls
    min_interval: Duration,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_call: Instant::now() - min_interval,
            min_interval,
        }
    }

    /// Check if allowed to proceed
    pub fn check(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_call) >= self.min_interval {
            self.last_call = now;
            true
        } else {
            false
        }
    }
}

/// Circular buffer
pub struct CircularBuffer<T> {
    /// Buffer storage
    buffer: Vec<Option<T>>,
    /// Head index
    head: usize,
    /// Tail index
    tail: usize,
    /// Current size
    size: usize,
}

impl<T: Clone> CircularBuffer<T> {
    /// Create new buffer with capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);
        
        Self {
            buffer,
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    /// Push element
    pub fn push(&mut self, item: T) -> Option<T> {
        let old = self.buffer[self.head].take();
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % self.buffer.len();
        
        if self.size == self.buffer.len() {
            self.tail = (self.tail + 1) % self.buffer.len();
        } else {
            self.size += 1;
        }
        
        old
    }

    /// Pop element
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        let item = self.buffer[self.tail].take();
        self.tail = (self.tail + 1) % self.buffer.len();
        self.size -= 1;
        item
    }

    /// Current size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Byte size human readable
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    
    format!("{:.2} {}", value, UNITS[exp])
}

/// Truncate string with ellipsis
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer() {
        let mut buf = CircularBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.pop(), Some(1));
    }

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(1024), "1.00 KB");
        assert_eq!(human_bytes(1024 * 1024), "1.00 MB");
    }
}
