//! Multi-vector support (ColBERT-style late interaction)
//!
//! Stores multiple vectors per document and performs MaxSim search.

use crate::distance::DistanceMetric;
use crate::error::{Error, Result};
use crate::types::InternalId;
use std::collections::HashMap;

/// Storage for multi-vector documents
pub struct MultiVectorStorage {
    /// Map from internal ID to list of vectors (flattened)
    /// vectors: [v1_1, v1_2, ..., v1_d, v2_1, ...]
    vectors: HashMap<InternalId, Vec<f32>>,
    /// Number of sub-vectors per document (if fixed) or store separately
    /// For simplicity, assuming variable number of vectors, so we need to know count or stride.
    /// Here we assume flattened and we need to know dimension to chunk it.
    dimensions: usize,
}

impl MultiVectorStorage {
    pub fn new(dimensions: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            dimensions,
        }
    }

    /// Insert a multi-vector document
    pub fn insert(&mut self, id: InternalId, vectors: Vec<Vec<f32>>) -> Result<()> {
        let mut flattened = Vec::with_capacity(vectors.len() * self.dimensions);
        for vec in vectors {
            if vec.len() != self.dimensions {
                return Err(Error::DimensionMismatch {
                    expected: self.dimensions,
                    got: vec.len(),
                });
            }
            flattened.extend(vec);
        }
        self.vectors.insert(id, flattened);
        Ok(())
    }

    /// MaxSim search
    /// Returns score = sum(max(sim(q_i, d_j))) for all q_i
    pub fn search(&self, query: &[Vec<f32>], metric: DistanceMetric) -> Vec<(InternalId, f32)> {
        let mut results = Vec::new();

        for (id, doc_vectors_flat) in &self.vectors {
            let mut score = 0.0;
            let doc_vectors: Vec<&[f32]> = doc_vectors_flat.chunks(self.dimensions).collect();

            for q in query {
                let mut max_sim = f32::NEG_INFINITY;
                for d in &doc_vectors {
                    let dist = metric.distance(q, d);
                    // Convert distance to similarity (1 - dist for cosine)
                    // Assuming cosine distance [0, 2] -> similarity [1, -1]
                    // Or usually we want high score = match.
                    // Let's use distance for now and minimize it?
                    // MaxSim usually maximizes dot product.
                    // If metric is Cosine Distance, sim = 1 - dist.
                    let sim = 1.0 - dist;
                    if sim > max_sim {
                        max_sim = sim;
                    }
                }
                score += max_sim;
            }
            results.push((*id, score));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_sim() {
        let mut storage = MultiVectorStorage::new(2);

        // Doc 1: [[1, 0], [0, 1]]
        storage
            .insert(InternalId::from(1), vec![vec![1.0, 0.0], vec![0.0, 1.0]])
            .unwrap();

        // Query: [[1, 0]]
        // Sim(q, d1) = 1.0 (match 1st vec)
        // Sim(q, d2) = 0.0 (orthogonal to 2nd vec)
        // Max = 1.0
        let query = vec![vec![1.0, 0.0]];
        let results = storage.search(&query, DistanceMetric::Cosine);

        assert_eq!(results[0].0, InternalId::from(1));
        assert!((results[0].1 - 1.0).abs() < 1e-5);
    }
}
