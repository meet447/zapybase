//! Benchmarks for PersistentVectorDb operations

#[cfg(feature = "persistence")]
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
#[cfg(feature = "persistence")]
use rand::{rngs::StdRng, Rng, SeedableRng};
#[cfg(feature = "persistence")]
use serde_json::{json, Value};
#[cfg(feature = "persistence")]
use surgedb_core::{DistanceMetric, PersistentConfig, PersistentVectorDb};
#[cfg(feature = "persistence")]
use surgedb_core::types::VectorId;
#[cfg(feature = "persistence")]
use tempfile::tempdir;

#[cfg(feature = "persistence")]
fn bench_sizes() -> Vec<usize> {
    let mut sizes = vec![1_000, 5_000];
    if std::env::var("SURGEDB_BENCH_LARGE").is_ok() {
        sizes.push(20_000);
    }
    sizes
}

#[cfg(feature = "persistence")]
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

#[cfg(feature = "persistence")]
fn bench_open_insert_checkpoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistent_db_open_insert_checkpoint");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            for sync_writes in [false, true].iter() {
                let items = generate_vectors(size, *dim, 42);
                group.bench_with_input(
                    BenchmarkId::new(format!("dim{dim}_sync{sync_writes}"), size),
                    &size,
                    |b, _| {
                        b.iter_batched(
                            || {
                                let dir = tempdir().expect("tempdir");
                                let mut config = PersistentConfig::default();
                                config.dimensions = *dim;
                                config.distance_metric = DistanceMetric::Cosine;
                                config.sync_writes = *sync_writes;
                                let db = PersistentVectorDb::open(dir.path(), config)
                                    .expect("open db");
                                (dir, db)
                            },
                            |(_dir, mut db)| {
                                for (id, vec, meta) in items.clone() {
                                    db.insert(id, &vec, meta).expect("insert");
                                }
                                db.checkpoint().expect("checkpoint");
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

#[cfg(feature = "persistence")]
fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("persistent_db_search");

    for dim in [128_usize, 384].iter() {
        for size in bench_sizes() {
            let dir = tempdir().expect("tempdir");
            let mut config = PersistentConfig::default();
            config.dimensions = *dim;
            config.distance_metric = DistanceMetric::Cosine;

            let mut db = PersistentVectorDb::open(dir.path(), config).expect("open db");
            let items = generate_vectors(size, *dim, 77);
            for (id, vec, meta) in items {
                db.insert(id, &vec, meta).expect("insert");
            }

            let mut rng = StdRng::seed_from_u64(123);
            let query: Vec<f32> = (0..*dim).map(|_| rng.gen::<f32>()).collect();

            group.bench_with_input(
                BenchmarkId::new(format!("dim{dim}"), size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let results = db.search(black_box(&query), 10, None).expect("search");
                        black_box(results.len());
                    });
                },
            );
        }
    }

    group.finish();
}

#[cfg(feature = "persistence")]
criterion_group!(benches, bench_open_insert_checkpoint, bench_search);
#[cfg(feature = "persistence")]
criterion_main!(benches);

#[cfg(not(feature = "persistence"))]
fn main() {}
