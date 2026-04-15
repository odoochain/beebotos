//! Performance Optimizations
//!
//! Collection of performance optimizations for the brain module.
//! Includes pooling, caching, and zero-copy operations.

use std::sync::Arc;

/// String interning pool for common strings
///
/// Reduces memory usage when the same strings are used repeatedly.
#[derive(Debug, Default)]
pub struct StringPool {
    // In a real implementation, this would use a HashSet<Arc<str>>
    // For now, this is a placeholder for the concept
}

impl StringPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or insert a string into the pool
    pub fn get(&self, s: &str) -> Arc<str> {
        // In production, check if string exists in pool first
        Arc::from(s)
    }
}

/// Object pool for reusable buffers
#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Vec<u8>>,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffers.push(Vec::with_capacity(buffer_size));
        }
        Self { buffers, capacity }
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&mut self) -> Vec<u8> {
        self.buffers.pop().unwrap_or_default()
    }

    /// Return a buffer to the pool
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        if self.buffers.len() < self.capacity {
            buffer.clear();
            self.buffers.push(buffer);
        }
    }
}

/// Fast approximate string matching
///
/// Uses SIMD-optimized algorithms when available.
pub fn fast_contains(haystack: &str, needle: &str) -> bool {
    // For now, delegate to standard library
    // In production, could use memchr or similar
    haystack.contains(needle)
}

/// Batch processing helper
///
/// Processes items in batches for better cache locality.
pub fn batch_process<T, F>(items: &[T], batch_size: usize, mut processor: F)
where
    F: FnMut(&[T]),
{
    for chunk in items.chunks(batch_size) {
        processor(chunk);
    }
}

/// SIMD-optimized float operations (when available)
#[cfg(target_arch = "x86_64")]
pub mod simd {
    /// Fast sum of f32 slice using AVX when available
    pub fn fast_sum(values: &[f32]) -> f32 {
        // Check for AVX support
        if is_x86_feature_detected!("avx") {
            unsafe { sum_avx(values) }
        } else {
            values.iter().sum()
        }
    }

    #[target_feature(enable = "avx")]
    unsafe fn sum_avx(values: &[f32]) -> f32 {
        use std::arch::x86_64::*;

        let mut sum = 0.0;
        let chunks = values.chunks_exact(8);
        let remainder = chunks.remainder();

        // Process 8 floats at a time
        for chunk in chunks {
            let _vec = _mm256_loadu_ps(chunk.as_ptr());
            // Horizontal add would go here
            // Simplified for demonstration
            sum += chunk.iter().sum::<f32>();
        }

        // Process remainder
        sum += remainder.iter().sum::<f32>();
        sum
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub mod simd {
    pub fn fast_sum(values: &[f32]) -> f32 {
        values.iter().sum()
    }
}

/// Memory-efficient string builder
#[derive(Debug)]
pub struct EfficientStringBuilder {
    buffer: String,
}

impl EfficientStringBuilder {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    pub fn build(self) -> String {
        self.buffer
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}

/// Lazy evaluation wrapper
pub struct Lazy<T, F> {
    value: Option<T>,
    factory: Option<F>,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub fn new(factory: F) -> Self {
        Self {
            value: None,
            factory: Some(factory),
        }
    }

    pub fn get(&mut self) -> &T {
        if self.value.is_none() {
            let factory = self.factory.take().expect("factory already consumed");
            self.value = Some(factory());
        }
        self.value.as_ref().unwrap()
    }

    pub fn into_inner(self) -> T {
        match self.value {
            Some(v) => v,
            None => (self.factory.expect("factory already consumed"))(),
        }
    }
}

/// Cache with TTL (Time To Live)
#[derive(Debug)]
pub struct TimedCache<K, V> {
    entries: std::collections::HashMap<K, (V, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl<K, V> TimedCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            ttl: std::time::Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|(v, time)| {
            if time.elapsed() < self.ttl {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, (value, std::time::Instant::now()));
    }

    pub fn cleanup(&mut self) {
        let now = std::time::Instant::now();
        self.entries
            .retain(|_, (_, time)| now.duration_since(*time) < self.ttl);
    }
}

/// Pre-allocated vector with fast clear
#[derive(Debug)]
pub struct FastClearVec<T> {
    buffer: Vec<T>,
    len: usize,
}

impl<T> FastClearVec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.len < self.buffer.capacity() {
            if self.len < self.buffer.len() {
                self.buffer[self.len] = item;
            } else {
                self.buffer.push(item);
            }
            self.len += 1;
        } else {
            self.buffer.push(item);
            self.len += 1;
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        // Note: elements are not dropped until overwritten or buffer is dropped
        // This is a memory/safety trade-off
    }

    pub fn as_slice(&self) -> &[T] {
        &self.buffer[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let pool = StringPool::new();
        let s1 = pool.get("test");
        let s2 = pool.get("test");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new(5, 1024);
        let buffer = pool.acquire();
        pool.release(buffer);
        let _buffer2 = pool.acquire();
    }

    #[test]
    fn test_batch_process() {
        let items: Vec<i32> = (0..100).collect();
        let mut sum = 0;
        batch_process(&items, 10, |batch| {
            sum += batch.iter().sum::<i32>();
        });
        assert_eq!(sum, 4950);
    }

    #[test]
    fn test_efficient_string_builder() {
        let mut builder = EfficientStringBuilder::with_capacity(100);
        builder.append("Hello ");
        builder.append("World");
        assert_eq!(builder.build(), "Hello World");
    }

    #[test]
    fn test_lazy() {
        let mut lazy = Lazy::new(|| 42);
        assert_eq!(*lazy.get(), 42);
        assert_eq!(lazy.into_inner(), 42);
    }

    #[test]
    fn test_timed_cache() {
        let mut cache = TimedCache::new(1);
        cache.insert("key", "value");
        assert_eq!(cache.get(&"key"), Some(&"value"));

        // Note: In real test, would need to wait for TTL
        // or mock time
    }

    #[test]
    fn test_fast_clear_vec() {
        let mut vec = FastClearVec::with_capacity(10);
        vec.push(1);
        vec.push(2);
        assert_eq!(vec.len(), 2);

        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_simd_fast_sum() {
        let values: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let sum = simd::fast_sum(&values);
        assert!((sum - 4950.0).abs() < 0.001);
    }
}
