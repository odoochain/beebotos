//! Optimized Memory Query Results
//!
//! Provides efficient, zero-copy query results using Arc<str> instead of String.

use std::sync::Arc;

/// Reference-counted string for memory-efficient results
pub type SharedString = Arc<str>;

/// Memory query results with minimal cloning
#[derive(Debug, Clone, Default)]
pub struct OptimizedMemoryResults {
    pub short_term: Vec<SharedString>,
    pub episodic: Vec<SharedString>,
    pub semantic: Vec<SharedString>,
    pub procedural: Vec<SharedString>,
}

impl OptimizedMemoryResults {
    /// Create empty results
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any results were found
    pub fn is_empty(&self) -> bool {
        self.short_term.is_empty()
            && self.episodic.is_empty()
            && self.semantic.is_empty()
            && self.procedural.is_empty()
    }

    /// Get total result count
    pub fn total_count(&self) -> usize {
        self.short_term.len() + self.episodic.len() + self.semantic.len() + self.procedural.len()
    }

    /// Convert to owned strings (when necessary)
    pub fn to_strings(&self) -> Vec<String> {
        let mut all = Vec::with_capacity(self.total_count());
        all.extend(self.short_term.iter().map(|s| s.to_string()));
        all.extend(self.episodic.iter().map(|s| s.to_string()));
        all.extend(self.semantic.iter().map(|s| s.to_string()));
        all.extend(self.procedural.iter().map(|s| s.to_string()));
        all
    }

    /// Merge all results into a single list of shared strings
    pub fn all_results(&self) -> Vec<SharedString> {
        let mut all = Vec::with_capacity(self.total_count());
        all.extend(self.short_term.iter().cloned());
        all.extend(self.episodic.iter().cloned());
        all.extend(self.semantic.iter().cloned());
        all.extend(self.procedural.iter().cloned());
        all
    }

    /// Get results as string slices
    pub fn as_strs(&self) -> Vec<&str> {
        let mut all = Vec::with_capacity(self.total_count());
        all.extend(self.short_term.iter().map(|s| s.as_ref()));
        all.extend(self.episodic.iter().map(|s| s.as_ref()));
        all.extend(self.semantic.iter().map(|s| s.as_ref()));
        all.extend(self.procedural.iter().map(|s| s.as_ref()));
        all
    }

    /// Convert from string results efficiently
    pub fn from_results(results: super::MemoryResults) -> Self {
        Self {
            short_term: results.short_term.into_iter().map(|s| s.into()).collect(),
            episodic: results.episodic.into_iter().map(|s| s.into()).collect(),
            semantic: results.semantic.into_iter().map(|s| s.into()).collect(),
            procedural: results.procedural.into_iter().map(|s| s.into()).collect(),
        }
    }
}

/// Builder for efficient memory results
#[derive(Debug, Default)]
pub struct MemoryResultsBuilder {
    short_term: Vec<SharedString>,
    episodic: Vec<SharedString>,
    semantic: Vec<SharedString>,
    procedural: Vec<SharedString>,
}

impl MemoryResultsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(short_term: usize, episodic: usize, semantic: usize, procedural: usize) -> Self {
        Self {
            short_term: Vec::with_capacity(short_term),
            episodic: Vec::with_capacity(episodic),
            semantic: Vec::with_capacity(semantic),
            procedural: Vec::with_capacity(procedural),
        }
    }

    pub fn add_short_term(&mut self, content: impl Into<SharedString>) {
        self.short_term.push(content.into());
    }

    pub fn add_episodic(&mut self, content: impl Into<SharedString>) {
        self.episodic.push(content.into());
    }

    pub fn add_semantic(&mut self, content: impl Into<SharedString>) {
        self.semantic.push(content.into());
    }

    pub fn add_procedural(&mut self, content: impl Into<SharedString>) {
        self.procedural.push(content.into());
    }

    pub fn build(self) -> OptimizedMemoryResults {
        OptimizedMemoryResults {
            short_term: self.short_term,
            episodic: self.episodic,
            semantic: self.semantic,
            procedural: self.procedural,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_results_empty() {
        let results = OptimizedMemoryResults::new();
        assert!(results.is_empty());
        assert_eq!(results.total_count(), 0);
    }

    #[test]
    fn test_builder_basic() {
        let mut builder = MemoryResultsBuilder::new();
        builder.add_short_term("test1");
        builder.add_episodic("test2");
        builder.add_semantic("test3");
        builder.add_procedural("test4");

        let results = builder.build();
        assert_eq!(results.total_count(), 4);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_builder_with_capacity() {
        let builder = MemoryResultsBuilder::with_capacity(10, 10, 5, 5);
        let results = builder.build();
        assert!(results.is_empty());
    }

    #[test]
    fn test_to_strings() {
        let mut builder = MemoryResultsBuilder::new();
        builder.add_short_term("test");
        let results = builder.build();
        
        let strings = results.to_strings();
        assert_eq!(strings, vec!["test"]);
    }

    #[test]
    fn test_as_strs() {
        let mut builder = MemoryResultsBuilder::new();
        builder.add_short_term("test1");
        builder.add_episodic("test2");
        let results = builder.build();
        
        let strs = results.as_strs();
        assert_eq!(strs, vec!["test1", "test2"]);
    }

    #[test]
    fn test_all_results() {
        let mut builder = MemoryResultsBuilder::new();
        builder.add_short_term("a");
        builder.add_short_term("b");
        let results = builder.build();
        
        let all = results.all_results();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_shared_string_efficiency() {
        let s1: SharedString = "test".into();
        let s2 = s1.clone();
        
        // Both should point to same data
        assert_eq!(s1.as_ptr(), s2.as_ptr());
    }
}
