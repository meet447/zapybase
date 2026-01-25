use surgedb_core::{PersistentConfig, PersistentVectorDb};
use tempfile::tempdir;

#[test]
fn test_delete_operation() {
    let dir = tempdir().unwrap();
    let config = PersistentConfig {
        dimensions: 4,
        ..Default::default()
    };

    let mut db = PersistentVectorDb::open(dir.path(), config).unwrap();

    // Insert vectors
    db.insert("vec1", &[1.0, 0.0, 0.0, 0.0], None).unwrap();
    db.insert("vec2", &[0.0, 1.0, 0.0, 0.0], None).unwrap();
    db.insert("vec3", &[0.0, 0.0, 1.0, 0.0], None).unwrap();

    assert_eq!(db.len(), 3);

    // Search before delete
    let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.as_str(), "vec1");

    // Delete vec1
    let deleted = db.delete("vec1").unwrap();
    assert!(deleted);

    // Search after delete (should find nothing or vec2/3)
    let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1, None).unwrap();
    // vec1 is gone. Next closest might be returned if we ask for k=1.
    // vec1 was [1, 0, 0, 0]. vec2 is [0, 1, 0, 0]. Distance is likely > 0.
    assert!(!results.is_empty());
    assert_ne!(results[0].0.as_str(), "vec1");

    // Try to delete again (should return false)
    let deleted = db.delete("vec1").unwrap();
    assert!(!deleted);
}

#[test]
fn test_delete_persistence() {
    let dir = tempdir().unwrap();
    let config = PersistentConfig {
        dimensions: 4,
        sync_writes: true,
        ..Default::default()
    };

    {
        let mut db = PersistentVectorDb::open(dir.path(), config.clone()).unwrap();
        db.insert("vec1", &[1.0, 0.0, 0.0, 0.0], None).unwrap();
        db.delete("vec1").unwrap();
    }

    // Reopen
    {
        let db = PersistentVectorDb::open(dir.path(), config).unwrap();
        // vec1 should be gone
        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1, None).unwrap();
        if !results.is_empty() {
            assert_ne!(results[0].0.as_str(), "vec1");
        }
    }
}
