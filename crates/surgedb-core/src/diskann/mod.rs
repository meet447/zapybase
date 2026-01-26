//! Disk-based vector indexing (DiskANN)
//!
//! Implements a graph-based index optimized for SSDs, allowing for datasets
//! that far exceed available RAM.
//!
//! ## Architecture
//!
//! 1. **Graph Index**: stored on disk as an adjacency list.
//! 2. **Vector Data**: stored on disk (mmap or direct IO).
//! 3. **Compressed Vectors**: stored in RAM (PQ) for navigation.
//! 4. **Search**:
//!    - Greedy search using in-memory compressed vectors to find candidates.
//!    - Fetch full-precision vectors from disk for final re-ranking.

pub mod layout;
pub mod storage;
pub mod vamana;
