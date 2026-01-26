//! Reciprocal Rank Fusion (RRF)
//!
//! Combines ranked results from multiple search algorithms (e.g., dense and sparse)
//! into a single unified ranking.

use crate::types::InternalId;
use std::collections::HashMap;

/// Perform Reciprocal Rank Fusion on two lists of results
///
/// Formula: score = sum(1 / (k + rank_i))
/// where k is a constant (typically 60)
pub fn reciprocal_rank_fusion(
    results_a: &[(InternalId, f32)],
    results_b: &[(InternalId, f32)],
    k_constant: f32,
    limit: usize,
) -> Vec<(InternalId, f32)> {
    let mut scores: HashMap<InternalId, f32> = HashMap::new();

    // Process list A
    for (rank, (id, _)) in results_a.iter().enumerate() {
        let score = 1.0 / (k_constant + (rank as f32) + 1.0);
        *scores.entry(*id).or_default() += score;
    }

    // Process list B
    for (rank, (id, _)) in results_b.iter().enumerate() {
        let score = 1.0 / (k_constant + (rank as f32) + 1.0);
        *scores.entry(*id).or_default() += score;
    }

    // Sort combined results
    let mut fused: Vec<_> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(limit);

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf() {
        let list_a = vec![
            (InternalId::from(1), 0.9),
            (InternalId::from(2), 0.8),
            (InternalId::from(3), 0.7),
        ];

        let list_b = vec![
            (InternalId::from(3), 0.9),
            (InternalId::from(1), 0.8),
            (InternalId::from(4), 0.7),
        ];

        // k = 1.0 for simple math
        // id 1: 1/(1+1) + 1/(1+2) = 0.5 + 0.333 = 0.833
        // id 2: 1/(1+2) = 0.333
        // id 3: 1/(1+3) + 1/(1+1) = 0.25 + 0.5 = 0.75
        // id 4: 1/(1+3) = 0.25

        let fused = reciprocal_rank_fusion(&list_a, &list_b, 1.0, 10);

        assert_eq!(fused[0].0, InternalId::from(1));
        assert_eq!(fused[1].0, InternalId::from(3));
        assert_eq!(fused[2].0, InternalId::from(2));
        assert_eq!(fused[3].0, InternalId::from(4));
    }
}
