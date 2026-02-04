//! Benchmarks for core VectorDb operations

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::{json, Value};
use surgedb_core::filter::Filter;
use surgedb_core::{Config, DistanceMetric, VectorDb};
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
            let tag = if i % 2 == 0 { "even" } else { "odd" };
            let metadata = json!({
                "tag": tag,
                "score": i as f64,
            });
            (VectorId::from(format!("vec_{i}")), vector, Some(metadata))
        })
        .collect()
}

fn build_db(dim: usize, count: usize, seed: u64) -> VectorDb {
    let config = Config {
        dimensions: dim,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let mut db = VectorDb::new(config).expect("create db");
    let items = generate_vectors(count, dim, seed);
    db.upsert_batch(items).expect("upsert batch");
    db
}

fn bench_insert_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_db_insert_single");

    for dim in [128_usize, 384].iter() {
        let vector = generate_vectors(1, *dim, 7).pop().unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter_batched(
                || VectorDb::new(Config {
                    dimensions: *dim,
                    distance_metric: DistanceMetric::Cosine,
                    ..Default::default()
                }).expect("create db"),
                |mut db| {
                    let (id, vec, meta) = vector.clone();
                    db.insert(id, &vec, meta).expect("insert");
                    black_box(db.len());
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_upsert_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_db_upsert_batch");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            let items = generate_vectors(size, *dim, 42);
            group.bench_with_input(
                BenchmarkId::new(format!("dim{dim}"), size),
                &size,
                |b, _| {
                    b.iter_batched(
                        || VectorDb::new(Config {
                            dimensions: *dim,
                            distance_metric: DistanceMetric::Cosine,
                            ..Default::default()
                        }).expect("create db"),
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

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_db_search");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            let db = build_db(*dim, size, 99);
            let mut rng = StdRng::seed_from_u64(123);
            let query: Vec<f32> = (0..*dim).map(|_| rng.gen::<f32>()).collect();

            group.bench_with_input(
                BenchmarkId::new(format!("dim{dim}"), size),
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

    group.finish();
}

fn bench_search_filtered(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_db_search_filtered");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            let db = build_db(*dim, size, 202);
            let mut rng = StdRng::seed_from_u64(456);
            let query: Vec<f32> = (0..*dim).map(|_| rng.gen::<f32>()).collect();
            let filter = Filter::Exact("tag".to_string(), json!("even"));

            group.bench_with_input(
                BenchmarkId::new(format!("dim{dim}"), size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let results = db
                            .search(black_box(&query), 10, Some(&filter))
                            .expect("search filtered");
                        black_box(results.len());
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_single,
    bench_upsert_batch,
    bench_search,
    bench_search_filtered
);
criterion_main!(benches);
