# SurgeDB Performance Benchmarks

This folder contains end-to-end performance scripts for the HTTP server.

## Server HTTP Benchmark

Start the server in another terminal:

```bash
cargo run --release -p surgedb-server
```

Run a mixed workload with prefill:

```bash
python3 scripts/perf/http_bench.py --mode mixed --duration 60 --concurrency 32 --prefill 20000 --dimensions 384
```

Search-only, with metadata filter:

```bash
python3 scripts/perf/http_bench.py --mode search --use-filter --duration 60 --concurrency 32 --prefill 20000
```

Quantized collection (SQ8):

```bash
python3 scripts/perf/http_bench.py --quantization SQ8 --mode search --duration 60 --concurrency 32 --prefill 20000
```

The script outputs a JSON summary with latency percentiles, QPS, and error counts.

## Core Microbenchmarks (Criterion)

Run all core benches:

```bash
cargo bench -p surgedb-core
```

Run specific benches:

```bash
cargo bench -p surgedb-core --bench vector_db
cargo bench -p surgedb-core --bench quantized_db
cargo bench -p surgedb-core --bench persistence
```

Enable larger dataset sizes:

```bash
SURGEDB_BENCH_LARGE=1 cargo bench -p surgedb-core --bench vector_db
```
