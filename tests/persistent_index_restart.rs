use dreamizdb::storage::persistence::PersistentTable;
use dreamizdb::storage::{Record, Table};
use std::fs;

fn test_paths(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_path = std::env::temp_dir().join(format!("dreamizdb-{name}.db"));
    let index_path = data_path.with_extension("idx");

    let _ = fs::remove_file(&data_path);
    let _ = fs::remove_file(&index_path);

    (data_path, index_path)
}

fn cleanup(data_path: &std::path::Path, index_path: &std::path::Path) {
    let _ = fs::remove_file(data_path);
    let _ = fs::remove_file(index_path);
}

fn sample_table() -> Table {
    let mut table = Table::new();

    table.insert(Record {
        id: 1,
        country: "BD".into(),
        value: 100.0,
    });

    table.insert(Record {
        id: 2,
        country: "IN".into(),
        value: 200.0,
    });

    table.insert(Record {
        id: 3,
        country: "BD".into(),
        value: 300.0,
    });

    table
}

#[test]
fn persistent_index_survives_restart() {
    let (data_path, index_path) = test_paths("persistent-index-restart");

    {
        let table = sample_table();

        let persistent = PersistentTable::create_from_table(&data_path, &table).unwrap();

        assert_eq!(persistent.page_count(), 3);
        assert_eq!(persistent.index_entry_count(), 2);
    }

    {
        let mut reopened = PersistentTable::open(&data_path).unwrap();

        let result = reopened.indexed_lookup("BD").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 3);
    }

    cleanup(&data_path, &index_path);
}

#[test]
fn persistent_index_and_sequential_scan_return_same_rows() {
    let (data_path, index_path) = test_paths("persistent-index-consistency");

    {
        let table = sample_table();
        PersistentTable::create_from_table(&data_path, &table).unwrap();
    }

    {
        let mut reopened = PersistentTable::open(&data_path).unwrap();

        let sequential = reopened.sequential_scan("BD").unwrap();
        reopened.clear_cache();

        let indexed = reopened.indexed_lookup("BD").unwrap();

        assert_eq!(sequential.len(), indexed.len());

        let sequential_ids: Vec<u64> = sequential.iter().map(|record| record.id).collect();
        let indexed_ids: Vec<u64> = indexed.iter().map(|record| record.id).collect();

        assert_eq!(sequential_ids, indexed_ids);
    }

    cleanup(&data_path, &index_path);
}

#[test]
fn rejects_index_for_modified_database() {
    let (data_path, index_path) = test_paths("index-mismatch");

    {
        let table = sample_table();
        PersistentTable::create_from_table(&data_path, &table).unwrap();
    }

    /*
     * Keep the original database but replace its index with metadata
     * containing an intentionally incorrect fingerprint.
     */
    {
        use dreamizdb::storage::index::PersistentIndex;
        use std::collections::BTreeMap;

        let mut stale_entries = BTreeMap::new();
        stale_entries.insert("BD".to_string(), vec![0, 2]);
        stale_entries.insert("IN".to_string(), vec![1]);

        PersistentIndex::create(
            &index_path,
            "intentionally-wrong-fingerprint".to_string(),
            "country",
            stale_entries,
        )
        .unwrap();
    }

    let result = PersistentTable::open(&data_path);

    let error = match result {
        Ok(_) => panic!("expected PersistentTable::open() to reject mismatched index"),
        Err(error) => error.to_string(),
    };

    assert!(
        error.contains("does not match database contents"),
        "unexpected error: {error}"
    );

    cleanup(&data_path, &index_path);
}
