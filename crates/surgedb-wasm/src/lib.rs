//! SurgeDB WebAssembly Bindings
//!
//! This crate provides JavaScript/TypeScript bindings for SurgeDB,
//! enabling in-browser vector search with sub-millisecond latency.
//!
//! # Example (JavaScript)
//! ```javascript
//! import init, { SurgeDB } from 'surgedb-wasm';
//!
//! await init();
//!
//! const db = new SurgeDB(384);  // 384 dimensions
//! db.insert("doc1", new Float32Array([...]), { title: "Hello" });
//!
//! const results = db.search(new Float32Array([...]), 10);
//! console.log(results);  // [{ id: "doc1", score: 0.95, metadata: {...} }, ...]
//! ```

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// =============================================================================
// Error Handling
// =============================================================================

#[derive(Debug)]
pub struct SurgeError(String);

impl From<surgedb_core::Error> for SurgeError {
    fn from(err: surgedb_core::Error) -> Self {
        SurgeError(err.to_string())
    }
}

impl From<SurgeError> for JsValue {
    fn from(err: SurgeError) -> Self {
        JsValue::from_str(&err.0)
    }
}

impl From<serde_json::Error> for SurgeError {
    fn from(err: serde_json::Error) -> Self {
        SurgeError(err.to_string())
    }
}

// =============================================================================
// Data Types
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub vector_count: usize,
    pub dimensions: usize,
    pub memory_usage_bytes: usize,
}

// =============================================================================
// Main Database Class
// =============================================================================

/// SurgeDB - High-performance vector database for the browser
///
/// Create an in-memory vector database optimized for semantic search.
#[wasm_bindgen]
pub struct SurgeDB {
    inner: surgedb_core::VectorDb,
}

#[wasm_bindgen]
impl SurgeDB {
    /// Create a new in-memory vector database
    ///
    /// @param dimensions - The dimensionality of vectors (e.g., 384 for MiniLM, 768 for BERT)
    #[wasm_bindgen(constructor)]
    pub fn new(dimensions: u32) -> Result<SurgeDB, JsValue> {
        let config = surgedb_core::Config {
            dimensions: dimensions as usize,
            distance_metric: surgedb_core::DistanceMetric::Cosine,
            ..Default::default()
        };

        let inner = surgedb_core::VectorDb::new(config).map_err(|e| SurgeError::from(e))?;

        Ok(SurgeDB { inner })
    }

    /// Insert a vector with optional metadata
    ///
    /// @param id - Unique identifier for the vector
    /// @param vector - Float32Array of the embedding
    /// @param metadata - Optional JSON metadata object
    #[wasm_bindgen]
    pub fn insert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: JsValue,
    ) -> Result<(), JsValue> {
        let meta = if metadata.is_undefined() || metadata.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(metadata)?)
        };

        self.inner
            .insert(id, &vector, meta)
            .map_err(|e| SurgeError::from(e))?;

        Ok(())
    }

    /// Insert or update a vector
    ///
    /// @param id - Unique identifier for the vector
    /// @param vector - Float32Array of the embedding
    /// @param metadata - Optional JSON metadata object
    #[wasm_bindgen]
    pub fn upsert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: JsValue,
    ) -> Result<(), JsValue> {
        let meta = if metadata.is_undefined() || metadata.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(metadata)?)
        };

        self.inner
            .upsert(id, &vector, meta)
            .map_err(|e| SurgeError::from(e))?;

        Ok(())
    }

    /// Delete a vector by ID
    ///
    /// @param id - The ID of the vector to delete
    /// @returns true if the vector was found and deleted
    #[wasm_bindgen]
    pub fn delete(&mut self, id: String) -> Result<bool, JsValue> {
        self.inner
            .delete(id)
            .map_err(|e| SurgeError::from(e).into())
    }

    /// Search for the k nearest neighbors
    ///
    /// @param query - Float32Array query vector
    /// @param k - Number of results to return
    /// @returns Array of { id, score, metadata } objects
    #[wasm_bindgen]
    pub fn search(&self, query: Vec<f32>, k: u32) -> Result<JsValue, JsValue> {
        let results = self
            .inner
            .search(&query, k as usize, None)
            .map_err(|e| SurgeError::from(e))?;

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|(id, score, metadata)| SearchResult {
                id: id.to_string(),
                score,
                metadata,
            })
            .collect();

        serde_wasm_bindgen::to_value(&search_results).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the number of vectors in the database
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the database is empty
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get database statistics
    #[wasm_bindgen]
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let stats = Stats {
            vector_count: self.inner.len(),
            dimensions: self.inner.config().dimensions,
            memory_usage_bytes: self.inner.memory_usage(),
        };

        serde_wasm_bindgen::to_value(&stats).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// =============================================================================
// Quantized Database (for larger datasets)
// =============================================================================

/// SurgeDB with SQ8 quantization for 4x memory reduction
#[wasm_bindgen]
pub struct SurgeDBQuantized {
    inner: surgedb_core::QuantizedVectorDb,
}

#[wasm_bindgen]
impl SurgeDBQuantized {
    /// Create a new quantized vector database (4x memory reduction)
    ///
    /// @param dimensions - The dimensionality of vectors
    #[wasm_bindgen(constructor)]
    pub fn new(dimensions: u32) -> Result<SurgeDBQuantized, JsValue> {
        let config = surgedb_core::QuantizedConfig {
            dimensions: dimensions as usize,
            distance_metric: surgedb_core::DistanceMetric::Cosine,
            quantization: surgedb_core::QuantizationType::SQ8,
            ..Default::default()
        };

        let inner =
            surgedb_core::QuantizedVectorDb::new(config).map_err(|e| SurgeError::from(e))?;

        Ok(SurgeDBQuantized { inner })
    }

    /// Insert a vector with optional metadata
    #[wasm_bindgen]
    pub fn insert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: JsValue,
    ) -> Result<(), JsValue> {
        let meta = if metadata.is_undefined() || metadata.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(metadata)?)
        };

        self.inner
            .insert(id, &vector, meta)
            .map_err(|e| SurgeError::from(e))?;

        Ok(())
    }

    /// Insert or update a vector
    #[wasm_bindgen]
    pub fn upsert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: JsValue,
    ) -> Result<(), JsValue> {
        let meta = if metadata.is_undefined() || metadata.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(metadata)?)
        };

        self.inner
            .upsert(id, &vector, meta)
            .map_err(|e| SurgeError::from(e))?;

        Ok(())
    }

    /// Delete a vector by ID
    #[wasm_bindgen]
    pub fn delete(&mut self, id: String) -> Result<bool, JsValue> {
        self.inner
            .delete(id)
            .map_err(|e| SurgeError::from(e).into())
    }

    /// Search for the k nearest neighbors
    #[wasm_bindgen]
    pub fn search(&self, query: Vec<f32>, k: u32) -> Result<JsValue, JsValue> {
        let results = self
            .inner
            .search(&query, k as usize, None)
            .map_err(|e| SurgeError::from(e))?;

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|(id, score, metadata)| SearchResult {
                id: id.to_string(),
                score,
                metadata,
            })
            .collect();

        serde_wasm_bindgen::to_value(&search_results).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the number of vectors
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get compression ratio
    #[wasm_bindgen(js_name = compressionRatio)]
    pub fn compression_ratio(&self) -> f32 {
        self.inner.compression_ratio()
    }

    /// Get statistics
    #[wasm_bindgen]
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let stats = Stats {
            vector_count: self.inner.len(),
            dimensions: self.inner.config().dimensions,
            memory_usage_bytes: self.inner.memory_usage(),
        };

        serde_wasm_bindgen::to_value(&stats).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Get the SurgeDB version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Log a message to the browser console
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}
