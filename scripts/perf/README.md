# SurgeDB Performance Benchmarks

This folder contains end-to-end performance scripts for the HTTP server and a perf harness.

## Perf Harness

Runs server + HTTP scenarios + core benches from a config file.

```bash
python3 scripts/perf/run_perf.py --config scripts/perf/perf.toml --output scripts/perf/perf_report.json
```

Skip core benches:

```bash
python3 scripts/perf/run_perf.py --skip-core
```

## HTTP Benchmark

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
python3 scripts/perf/http_bench.py --mode search --use-filter --filter-type Exact --duration 60 --concurrency 32 --prefill 20000
```

Search-only, exclude metadata from responses:

```bash
python3 scripts/perf/http_bench.py --mode search --no-metadata --duration 60 --concurrency 32 --prefill 20000
```

Save results to a JSON file:

```bash
python3 scripts/perf/http_bench.py --mode search --duration 60 --output /tmp/perf.json
```

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
