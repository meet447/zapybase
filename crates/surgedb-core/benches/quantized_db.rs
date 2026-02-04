//! Benchmarks for QuantizedVectorDb operations

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::{json, Value};
use surgedb_core::{QuantizationType, QuantizedConfig, QuantizedVectorDb, DistanceMetric};
use surgedb_core::types::VectorId;

fn bench_sizes() -> Vec<usize> {
    let mut sizes = vec![2_000, 10_000];
    if std::env::var("SURGEDB_BENCH_LARGE").is_ok() {
        sizes.push(50_000);
    }
    sizes
}

fn generate_vectors(
    count: usize,
    dim: usize,
    seed: u64,
) -> Vec<(VectorId, Vec<f32>, Option<Value>)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|i| {
            let vector: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
            let metadata = json!({
                "tag": if i % 2 == 0 { "even" } else { "odd" },
                "score": i as f64,
            });
            (VectorId::from(format!("vec_{i}")), vector, Some(metadata))
        })
        .collect()
}

fn build_db(dim: usize, count: usize, seed: u64, quantization: QuantizationType) -> QuantizedVectorDb {
    let config = QuantizedConfig {
        dimensions: dim,
        distance_metric: DistanceMetric::Cosine,
        quantization,
        ..Default::default()
    };
    let mut db = QuantizedVectorDb::new(config).expect("create quantized db");
    let items = generate_vectors(count, dim, seed);
    db.upsert_batch(items).expect("upsert batch");
    db
}

fn bench_upsert_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantized_db_upsert_batch");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            for quant in [QuantizationType::SQ8, QuantizationType::Binary].iter() {
                let items = generate_vectors(size, *dim, 42);
                group.bench_with_input(
                    BenchmarkId::new(format!("dim{dim}_{quant:?}"), size),
                    &size,
                    |b, _| {
                        b.iter_batched(
                            || QuantizedVectorDb::new(QuantizedConfig {
                                dimensions: *dim,
                                distance_metric: DistanceMetric::Cosine,
                                quantization: *quant,
                                ..Default::default()
                            }).expect("create quantized db"),
                            |mut db| {
                                db.upsert_batch(items.clone()).expect("upsert batch");
                                black_box(db.len());
                            },
                            BatchSize::SmallInput,
                        );
                    },
                );
            }
        }
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantized_db_search");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            for quant in [QuantizationType::SQ8, QuantizationType::Binary].iter() {
                let db = build_db(*dim, size, 99, *quant);
                let mut rng = StdRng::seed_from_u64(123);
                let query: Vec<f32> = (0..*dim).map(|_| rng.gen::<f32>()).collect();

                group.bench_with_input(
                    BenchmarkId::new(format!("dim{dim}_{quant:?}"), size),
                    &size,
                    |b, _| {
                        b.iter(|| {
                            let results = db
                                .search(black_box(&query), 10, None)
                                .expect("search");
                            black_box(results.len());
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_upsert_batch, bench_search);
criterion_main!(benches);
