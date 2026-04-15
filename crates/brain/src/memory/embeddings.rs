//! Embedding Module
//!
//! Vector embeddings for semantic similarity search.

#![allow(dead_code)]

/// Embedding client (placeholder for FastEmbed integration)
#[derive(Debug, Clone)]
pub struct EmbeddingClient {
    dimension: usize,
    cache: std::collections::HashMap<String, Vec<f32>>,
}

/// Embedding vector
pub type Embedding = Vec<f32>;

impl EmbeddingClient {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            cache: std::collections::HashMap::new(),
        }
    }

    /// Generate embedding for text (placeholder)
    pub fn embed(&mut self, text: impl AsRef<str>) -> Embedding {
        let text = text.as_ref();

        // Check cache
        if let Some(embedding) = self.cache.get(text) {
            return embedding.clone();
        }

        // Generate simple hash-based embedding (placeholder)
        // In real implementation, this would use FastEmbed or similar
        let embedding = self.simple_hash_embedding(text);

        self.cache.insert(text.to_string(), embedding.clone());
        embedding
    }

    /// Compute cosine similarity
    pub fn similarity(&self, a: &Embedding, b: &Embedding) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Find most similar
    pub fn find_most_similar(
        &self,
        query: &Embedding,
        candidates: &[(String, Embedding)],
        top_k: usize,
    ) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = candidates
            .iter()
            .map(|(id, emb)| (id.clone(), self.similarity(query, emb)))
            .collect();

        similarities.sort_by(|a, b| crate::utils::compare_f32(&b.1, &a.1));
        similarities.into_iter().take(top_k).collect()
    }

    /// Simple hash-based embedding (placeholder for real embedding model)
    fn simple_hash_embedding(&self, text: &str) -> Embedding {
        let mut embedding = vec![0.0; self.dimension];
        let bytes = text.as_bytes();

        for (i, byte) in bytes.iter().enumerate() {
            let idx = i % self.dimension;
            embedding[idx] += (*byte as f32) / 255.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }
}

impl Default for EmbeddingClient {
    fn default() -> Self {
        Self::new(384) // Default dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        let client = EmbeddingClient::new(4);
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0, 0.0];

        assert!((client.similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!(client.similarity(&a, &c) < 0.001);
    }
}
