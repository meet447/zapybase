//! Sparse vector indexing for hybrid search
//!
//! Implements inverted index for sparse vectors (e.g., BM25 or SPLADE).

use crate::types::InternalId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sparse vector representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SparseVector {
    /// Indices of non-zero elements (sorted)
    pub indices: Vec<u32>,
    /// Values of non-zero elements
    pub values: Vec<f32>,
}

impl SparseVector {
    /// Create a new sparse vector from pairs (must be sorted by index)
    pub fn new(mut pairs: Vec<(u32, f32)>) -> Self {
        pairs.sort_by_key(|(idx, _)| *idx);
        let (indices, values): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        Self { indices, values }
    }

    /// Calculate dot product with another sparse vector
    pub fn dot(&self, other: &SparseVector) -> f32 {
        let mut sum = 0.0;
        let mut i = 0;
        let mut j = 0;

        while i < self.indices.len() && j < other.indices.len() {
            let idx_i = self.indices[i];
            let idx_j = other.indices[j];

            if idx_i == idx_j {
                sum += self.values[i] * other.values[j];
                i += 1;
                j += 1;
            } else if idx_i < idx_j {
                i += 1;
            } else {
                j += 1;
            }
        }
        sum
    }
}

/// Inverted index for fast sparse vector search
#[derive(Default, Serialize, Deserialize)]
pub struct InvertedIndex {
    /// Map from token index to list of (document ID, score)
    postings: HashMap<u32, Vec<(InternalId, f32)>>,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a sparse vector to the index
    pub fn insert(&mut self, id: InternalId, vector: &SparseVector) {
        for (&idx, &val) in vector.indices.iter().zip(vector.values.iter()) {
            self.postings.entry(idx).or_default().push((id, val));
        }
    }

    /// Remove a document from the index (expensive)
    pub fn remove(&mut self, id: InternalId, vector: &SparseVector) {
        for &idx in &vector.indices {
            if let Some(list) = self.postings.get_mut(&idx) {
                if let Some(pos) = list.iter().position(|(doc_id, _)| *doc_id == id) {
                    list.swap_remove(pos);
                }
            }
        }
    }

    /// Search for documents matching the sparse query
    pub fn search(&self, query: &SparseVector, k: usize) -> Vec<(InternalId, f32)> {
        let mut scores: HashMap<InternalId, f32> = HashMap::new();

        // Accumulate scores
        for (&token_idx, &query_val) in query.indices.iter().zip(query.values.iter()) {
            if let Some(posting_list) = self.postings.get(&token_idx) {
                for (doc_id, doc_val) in posting_list {
                    *scores.entry(*doc_id).or_default() += query_val * doc_val;
                }
            }
        }

        // Convert to sorted list
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_dot_product() {
        let v1 = SparseVector::new(vec![(1, 0.5), (3, 0.8), (5, 0.2)]);
        let v2 = SparseVector::new(vec![(1, 0.5), (2, 0.9), (3, 0.2)]);

        // 1: 0.5*0.5 = 0.25
        // 2: 0*0.9 = 0
        // 3: 0.8*0.2 = 0.16
        // 5: 0.2*0 = 0
        // Sum = 0.41
        let dot = v1.dot(&v2);
        assert!((dot - 0.41).abs() < 1e-5);
    }

    #[test]
    fn test_inverted_index() {
        let mut index = InvertedIndex::new();

        let v1 = SparseVector::new(vec![(1, 1.0), (2, 1.0)]);
        let v2 = SparseVector::new(vec![(2, 1.0), (3, 1.0)]);
        let v3 = SparseVector::new(vec![(1, 1.0), (3, 1.0)]);

        index.insert(InternalId::from(1), &v1);
        index.insert(InternalId::from(2), &v2);
        index.insert(InternalId::from(3), &v3);

        // Query: token 1 and 2
        let query = SparseVector::new(vec![(1, 1.0), (2, 1.0)]);
        let results = index.search(&query, 10);

        // v1: 1.0*1.0 + 1.0*1.0 = 2.0
        // v2: 1.0*1.0 = 1.0
        // v3: 1.0*1.0 = 1.0

        assert_eq!(results[0].0, InternalId::from(1));
        assert!((results[0].1 - 2.0).abs() < 1e-5);
    }
}
