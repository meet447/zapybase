# ⚡ ZappyBase

**ZappyBase** is a high-performance, super-fast, and ultra-lightweight vector database built from scratch in Rust. It is optimized for edge devices (like Mac M-series) and resource-constrained environments (like 1GB RAM cloud instances).

[![Rust CI](https://github.com/meetsonawane/zapybase/actions/workflows/rust.yml/badge.svg)](https://github.com/meetsonawane/zapybase/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🚀 Performance Snapshot

We validate every build for **Recall** (accuracy) and **Latency**.

| Mode | Recall @ 10 | Latency (Avg) | Compression |
| :--- | :--- | :--- | :--- |
| **HNSW (In-Memory)** | **98.5%** | 0.22 ms | 1x |
| **SQ8 (Quantized)** | **98.9%** | **0.16 ms** | **3.76x** |
| **Binary (1-bit)** | 25.8% | 0.23 ms | 32.0x |

---

## ✨ Features

- **Adaptive HNSW Indexing**: High-speed approximate nearest neighbor search.
- **SIMD Optimized**: Hand-tuned kernels for NEON (Apple Silicon) and AVX-512 (x86).
- **Plug-and-Play Quantization**: 
  - **SQ8**: 4x compression with <1% accuracy loss.
  - **Binary**: 32x compression for massive datasets.
- **ACID-Compliant Persistence**: Write-Ahead Log (WAL) and Snapshots for crash-safe data.
- **Mmap Support**: Disk-resident vectors for datasets larger than RAM.
- **Collections & Metadata**: Manage multiple collections with rich JSON metadata.
- **HTTP Server**: Built-in high-performance Axum server for easy deployment.

---

## 📦 Installation

Add ZappyBase to your `Cargo.toml`:

```toml
[dependencies]
zapybase-core = { git = "https://github.com/meet447/zapybase" }
```

---

## 🛠️ Quick Start (Rust)

```rust
use zapybase_core::{PersistentVectorDb, PersistentConfig, DistanceMetric};

fn main() {
    // 1. Setup Persistent Database
    let config = PersistentConfig {
        dimensions: 384, // MiniLM size
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let mut db = PersistentVectorDb::open("./zapybase_data", config).unwrap();

    // 2. Insert Vector with Metadata
    let vec = vec![0.1; 384];
    let meta = serde_json::json!({"title": "ZappyBase Guide"});
    db.insert("doc_1", &vec, Some(meta)).unwrap();

    // 3. Search
    let query = vec![0.1; 384];
    let results = db.search(&query, 5).unwrap();
    
    println!("Found match: {} (meta: {:?})", results[0].0, results[0].2);
}
```

---

## 🌐 HTTP Server

ZappyBase includes a high-performance HTTP server powered by **Axum**.

### Start the Server
```bash
cargo run --release -p zapybase-server
# Server listening on 0.0.0.0:3000
```

### API Usage

**Create Collection**
```bash
curl -X POST http://localhost:3000/collections \
  -H "Content-Type: application/json" \
  -d '{ "name": "docs", "dimensions": 384 }'
```

**Insert Vector**
```bash
curl -X POST http://localhost:3000/collections/docs/vectors \
  -H "Content-Type: application/json" \
  -d '{
    "id": "vec1",
    "vector": [0.1, 0.2, 0.3, ...],
    "metadata": { "category": "AI", "tags": ["fast"] }
  }'
```

**Search**
```bash
curl -X POST http://localhost:3000/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{ "vector": [0.1, 0.2, 0.3, ...], "k": 5 }'
```

---

## 🖥️ CLI Usage

ZappyBase comes with a powerful CLI for benchmarking and validation.

```bash
# Run the validation suite (Recall & Latency)
cargo run --release -- validate

# Benchmark with 10k vectors + SQ8 compression
cargo run --release -- bench -c 10000 -q sq8

# Test persistence and recovery
cargo run --release -- persist
```

---

## 🗺️ Roadmap

- [x] SIMD Distance Kernels (NEON/AVX)
- [x] HNSW Algorithm
- [x] SQ8 & Binary Quantization
- [x] WAL & Snapshot Persistence
- [x] Mmap Storage Backend
- [x] Collections & Metadata Support
- [x] HTTP Server (Axum)
- [ ] UniFFI Bindings (Python, Swift, Kotlin)
- [ ] Zero-Config RAG Pipeline (Candle Integration)
- [ ] Distributed Consensus (Raft)

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
