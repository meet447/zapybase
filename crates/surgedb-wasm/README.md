# SurgeDB WASM

WebAssembly bindings for SurgeDB - run high-performance vector search directly in the browser.

## Features

- **In-browser vector search**: No server required, runs entirely client-side
- **Sub-millisecond latency**: HNSW algorithm optimized for speed
- **Quantization support**: 4x memory reduction with SQ8
- **TypeScript support**: Full type definitions included
- **~200KB gzipped**: Lightweight bundle size

## Installation

### From npm (coming soon)

```bash
npm install surgedb-wasm
```

### From source

```bash
cd crates/surgedb-wasm
wasm-pack build --target web --release
```

## Usage

### Browser (ES Modules)

```javascript
import init, { SurgeDB, version } from './pkg/surgedb_wasm.js';

async function main() {
    // Initialize WASM module
    await init();
    
    console.log(`SurgeDB Version: ${version()}`);
    
    // Create database (384 dimensions for MiniLM embeddings)
    const db = new SurgeDB(384);
    
    // Insert vectors with metadata
    db.insert("doc1", new Float32Array([0.1, 0.2, ...]), { title: "Hello World" });
    db.insert("doc2", new Float32Array([0.3, 0.4, ...]), { title: "Goodbye World" });
    
    // Search for similar vectors
    const results = db.search(new Float32Array([0.1, 0.2, ...]), 5);
    
    for (const result of results) {
        console.log(`${result.id}: ${result.score} - ${result.metadata.title}`);
    }
    
    // Get stats
    console.log(db.stats());
    // { vector_count: 2, dimensions: 384, memory_usage_bytes: 3456 }
    
    // Clean up when done
    db.free();
}

main();
```

### With Quantization (4x memory reduction)

```javascript
import init, { SurgeDBQuantized } from './pkg/surgedb_wasm.js';

await init();

// Quantized database uses 4x less memory
const db = new SurgeDBQuantized(384);

db.insert("doc1", embedding, { category: "tech" });

const results = db.search(query, 10);

console.log(`Compression ratio: ${db.compressionRatio()}x`);
```

## API Reference

### SurgeDB

| Method | Description |
|--------|-------------|
| `new SurgeDB(dimensions)` | Create new database |
| `insert(id, vector, metadata)` | Insert vector with metadata |
| `upsert(id, vector, metadata)` | Insert or update |
| `delete(id)` | Delete vector by ID |
| `search(query, k)` | Find k nearest neighbors |
| `len()` | Get vector count |
| `isEmpty()` | Check if empty |
| `stats()` | Get statistics |
| `free()` | Release memory |

### SurgeDBQuantized

Same API as `SurgeDB`, plus:

| Method | Description |
|--------|-------------|
| `compressionRatio()` | Get memory compression ratio |

## Performance

Benchmarks on M2 MacBook Air (Chrome):

| Operation | 1,000 vectors | Time |
|-----------|---------------|------|
| Insert | 384 dim | ~50ms |
| Search (k=10) | Per query | ~0.5ms |
| Memory | Standard | ~1.5MB |
| Memory | Quantized (SQ8) | ~400KB |

## Building

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for browser
cd crates/surgedb-wasm
wasm-pack build --target web --release

# Build for Node.js
wasm-pack build --target nodejs --release

# Build for bundlers (webpack, etc.)
wasm-pack build --target bundler --release
```

## Demo

Open `index.html` in a browser after building:

```bash
cd crates/surgedb-wasm
wasm-pack build --target web --release
python3 -m http.server 8080
# Open http://localhost:8080
```

## Use Cases

- **Offline-first AI apps**: Semantic search without internet
- **Privacy-preserving search**: Data never leaves the browser
- **PWA with semantic search**: Add vector search to any web app
- **Browser extensions**: Content search and similarity matching
- **Edge computing**: Run on any device with a browser

## Bundle Size

| Component | Size (gzipped) |
|-----------|----------------|
| WASM binary | ~80KB |
| JS bindings | ~10KB |
| **Total** | **~90KB** |
