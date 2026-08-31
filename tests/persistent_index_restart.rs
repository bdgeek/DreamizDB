use dreamizdb::benchmark::sample_table;
use dreamizdb::storage::persistence::PersistentTable;

#[test]
fn persistent_index_survives_restart_without_full_scan() {
    let path = std::env::temp_dir().join("dreamizdb-restart-index.db");
    let table = sample_table(2_000);

    {
        let _ = PersistentTable::create_from_table(&path, &table).unwrap();
    }

    let mut reopened = PersistentTable::open(&path).unwrap();
    assert!(reopened.index_entry_count() > 0);
    reopened.reset_metrics();
    let rows = reopened.indexed_lookup("BD").unwrap();
    let reads = reopened.io_stats().reads;

    assert!(!rows.is_empty());
    assert!(reads < reopened.page_count());

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("idx"));
}
