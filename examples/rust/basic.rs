use serde_json::json;
use surgedb_core::{Config, DistanceMetric, VectorDb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize a new in-memory database
    let config = Config {
        dimensions: 3,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let mut db = VectorDb::new(config)?;

    println!("SurgeDB Rust Example");
    println!("-------------------");

    // 2. Insert some vectors with metadata
    db.insert(
        "apple",
        &[1.0, 0.0, 0.0],
        Some(json!({"type": "fruit", "color": "red"})),
    )?;
    db.insert(
        "banana",
        &[0.0, 1.0, 0.0],
        Some(json!({"type": "fruit", "color": "yellow"})),
    )?;
    db.insert(
        "truck",
        &[0.0, 0.0, 1.0],
        Some(json!({"type": "vehicle", "color": "blue"})),
    )?;

    println!("Inserted 3 vectors.");

    // 3. Search for the most similar vector
    let query = [0.9, 0.1, 0.0];
    println!("\nSearching for: {:?}", query);

    let results = db.search(&query, 1, None)?;

    if let Some((id, distance, metadata)) = results.first() {
        println!("Match Found!");
        println!("ID: {}", id);
        println!("Distance: {:.4}", distance);
        println!("Metadata: {}", metadata.as_ref().unwrap());
    }

    Ok(())
}
