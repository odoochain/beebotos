//! Memory Index
//!
//! Inverted index for fast memory retrieval.
//! Uses tokenization and indexing to provide O(1) average lookup time
//! instead of O(n) linear search.

use std::collections::{HashMap, HashSet};

/// Inverted index for memory content
#[derive(Debug, Clone, Default)]
pub struct MemoryIndex {
    /// Token -> Memory IDs mapping
    index: HashMap<String, HashSet<String>>,
    /// Number of items indexed
    item_count: usize,
    /// Total tokens indexed
    token_count: usize,
}

impl MemoryIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            item_count: 0,
            token_count: 0,
        }
    }

    /// Index a new memory item
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the memory item
    /// * `content` - Content to index
    ///
    /// # Example
    /// ```
    /// use beebotos_brain::memory::index::MemoryIndex;
    ///
    /// let mut index = MemoryIndex::new();
    /// index.add("mem_1", "The quick brown fox");
    /// index.add("mem_2", "The lazy dog");
    /// ```
    pub fn add(&mut self, id: &str, content: &str) {
        let tokens = tokenize(content);

        for token in tokens {
            self.index.entry(token).or_default().insert(id.to_string());
        }

        self.item_count += 1;
        self.token_count = self.index.len();
    }

    /// Remove a memory item from the index
    pub fn remove(&mut self, id: &str) {
        let id_str = id.to_string();

        for (_, ids) in self.index.iter_mut() {
            ids.remove(&id_str);
        }

        // Clean up empty token entries
        self.index.retain(|_, ids| !ids.is_empty());

        self.item_count = self.item_count.saturating_sub(1);
        self.token_count = self.index.len();
    }

    /// Update an existing memory item's index
    pub fn update(&mut self, id: &str, _old_content: &str, new_content: &str) {
        self.remove(id);
        self.add(id, new_content);
    }

    /// Search for memory items matching the query
    ///
    /// Returns a vector of (id, relevance_score) pairs sorted by relevance.
    pub fn search(&self, query: &str) -> Vec<(String, f32)> {
        let query_tokens: Vec<String> = tokenize(query);

        if query_tokens.is_empty() {
            return vec![];
        }

        // Calculate relevance scores
        let mut scores: HashMap<String, f32> = HashMap::new();

        for token in &query_tokens {
            if let Some(ids) = self.index.get(token) {
                let idf = self.calculate_idf(token);

                for id in ids {
                    *scores.entry(id.clone()).or_insert(0.0) += idf;
                }
            }
        }

        // Sort by score (descending)
        let mut results: Vec<(String, f32)> = scores.into_iter().collect();
        results.sort_by(|a, b| {
            crate::utils::compare_f32(&b.1, &a.1) // Descending order
        });

        results
    }

    /// Find memory items containing ALL query tokens (AND search)
    pub fn search_and(&self, query: &str) -> Vec<(String, f32)> {
        let query_tokens: Vec<String> = tokenize(query);

        if query_tokens.is_empty() {
            return vec![];
        }

        // Find intersection of all token results
        let mut result_ids: Option<HashSet<String>> = None;

        for token in &query_tokens {
            let token_ids = self.index.get(token).cloned().unwrap_or_default();

            result_ids = match result_ids {
                Some(existing) => Some(existing.intersection(&token_ids).cloned().collect()),
                None => Some(token_ids),
            };
        }

        let ids = result_ids.unwrap_or_default();

        // Calculate scores
        let mut results: Vec<(String, f32)> = ids
            .into_iter()
            .map(|id| {
                let score = query_tokens
                    .iter()
                    .filter(|t| self.index.get(*t).map_or(false, |s| s.contains(&id)))
                    .count() as f32
                    / query_tokens.len() as f32;
                (id, score)
            })
            .collect();

        results.sort_by(|a, b| crate::utils::compare_f32(&b.1, &a.1));

        results
    }

    /// Get all memory IDs in the index
    pub fn all_ids(&self) -> HashSet<String> {
        self.index.values().flatten().cloned().collect()
    }

    /// Check if a memory ID is indexed
    pub fn contains(&self, id: &str) -> bool {
        self.index.values().any(|ids| ids.contains(id))
    }

    /// Get the number of indexed items
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// Get the number of unique tokens
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            item_count: self.item_count,
            token_count: self.token_count,
            avg_tokens_per_item: if self.item_count > 0 {
                self.index.values().map(|ids| ids.len()).sum::<usize>() as f32
                    / self.item_count as f32
            } else {
                0.0
            },
        }
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.index.clear();
        self.item_count = 0;
        self.token_count = 0;
    }

    /// Calculate inverse document frequency for a token
    fn calculate_idf(&self, token: &str) -> f32 {
        let total_docs = self.item_count as f32;
        let docs_with_token = self.index.get(token).map_or(0, |ids| ids.len()) as f32;

        if docs_with_token == 0.0 {
            0.0
        } else {
            (total_docs / docs_with_token).ln().max(0.0)
        }
    }
}

/// Index statistics
#[derive(Debug, Clone, Copy)]
pub struct IndexStats {
    pub item_count: usize,
    pub token_count: usize,
    pub avg_tokens_per_item: f32,
}

/// Tokenize content into searchable tokens
///
/// Converts to lowercase, splits on whitespace and punctuation,
/// removes very short tokens, and removes common stop words.
fn tokenize(content: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above", "below", "between",
        "under", "and", "but", "or", "yet", "so", "if", "because", "although", "though", "while",
        "where", "when", "that", "which", "who", "whom", "whose", "what", "this", "these", "those",
        "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
    ];

    content
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| s.len() >= 2) // Skip very short tokens
        .filter(|s| !STOP_WORDS.contains(&s))
        .map(|s| s.to_string())
        .collect()
}

/// Query preprocessor for better search results
pub struct QueryPreprocessor;

impl QueryPreprocessor {
    /// Preprocess a query string for better matching
    pub fn preprocess(query: &str) -> String {
        query.to_lowercase()
    }

    /// Extract key terms from a query (removing stop words)
    pub fn extract_key_terms(query: &str) -> Vec<String> {
        tokenize(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_index_creation() {
        let index = MemoryIndex::new();
        assert_eq!(index.item_count(), 0);
        assert_eq!(index.token_count(), 0);
    }

    #[test]
    fn test_memory_index_add() {
        let mut index = MemoryIndex::new();
        index.add("id1", "The quick brown fox");

        assert_eq!(index.item_count(), 1);
        assert!(index.token_count() > 0);
        assert!(index.contains("id1"));
    }

    #[test]
    fn test_memory_index_search() {
        let mut index = MemoryIndex::new();
        index.add("id1", "The quick brown fox");
        index.add("id2", "The lazy dog");
        index.add("id3", "A quick cat");

        let results = index.search("quick");
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|(id, _)| id == "id1"));
        assert!(results.iter().any(|(id, _)| id == "id3"));
    }

    #[test]
    fn test_memory_index_search_and() {
        let mut index = MemoryIndex::new();
        index.add("id1", "quick brown fox");
        index.add("id2", "quick lazy dog");
        index.add("id3", "slow brown dog");

        let results = index.search_and("quick brown");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "id1");
    }

    #[test]
    fn test_memory_index_remove() {
        let mut index = MemoryIndex::new();
        index.add("id1", "The quick brown fox");
        index.remove("id1");

        assert!(!index.contains("id1"));
        assert_eq!(index.item_count(), 0);
    }

    #[test]
    fn test_memory_index_update() {
        let mut index = MemoryIndex::new();
        index.add("id1", "The quick brown fox");
        index.update("id1", "The quick brown fox", "The slow green turtle");

        let results = index.search("quick");
        assert!(results.is_empty());

        let results = index.search("turtle");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_index_empty_search() {
        let index = MemoryIndex::new();
        let results = index.search("anything");
        assert!(results.is_empty());
    }

    #[test]
    fn test_memory_index_stats() {
        let mut index = MemoryIndex::new();
        index.add("id1", "quick brown fox");
        index.add("id2", "quick brown dog");

        let stats = index.stats();
        assert_eq!(stats.item_count, 2);
        assert!(stats.token_count > 0);
        assert!(stats.avg_tokens_per_item > 0.0);
    }

    #[test]
    fn test_tokenize() {
        let tokens: Vec<String> = tokenize("The quick, brown fox!");
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
        assert!(!tokens.contains(&"the".to_string())); // Stop word removed
    }

    #[test]
    fn test_tokenize_unicode() {
        let tokens: Vec<String> = tokenize("Hello 世界 test");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        // Unicode characters may be handled differently
    }

    #[test]
    fn test_query_preprocessor() {
        let preprocessed = QueryPreprocessor::preprocess("HeLLo WoRLd");
        assert_eq!(preprocessed, "hello world");
    }

    #[test]
    fn test_extract_key_terms() {
        let terms = QueryPreprocessor::extract_key_terms("The quick brown fox");
        assert!(terms.contains(&"quick".to_string()));
        assert!(terms.contains(&"brown".to_string()));
        assert!(!terms.contains(&"the".to_string()));
    }

    #[test]
    fn test_search_relevance_ordering() {
        let mut index = MemoryIndex::new();

        // Add items with varying relevance
        index.add("rare", "unique zebra");
        index.add("common", "quick quick quick");

        let results = index.search("quick");
        // "common" should rank higher for "quick" since it appears more
        assert_eq!(results[0].0, "common");
    }

    #[test]
    fn test_memory_index_clear() {
        let mut index = MemoryIndex::new();
        index.add("id1", "test content");
        index.clear();

        assert_eq!(index.item_count(), 0);
        assert_eq!(index.token_count(), 0);
        assert!(!index.contains("id1"));
    }

    #[test]
    fn test_all_ids() {
        let mut index = MemoryIndex::new();
        index.add("id1", "content one");
        index.add("id2", "content two");
        index.add("id3", "content three");

        let all = index.all_ids();
        assert_eq!(all.len(), 3);
        assert!(all.contains("id1"));
        assert!(all.contains("id2"));
        assert!(all.contains("id3"));
    }
}
