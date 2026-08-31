use dreamizdb::benchmark::{benchmark_persistent_index, benchmark_persistent_scan, sample_table};
use dreamizdb::storage::persistence::PersistentTable;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn persistent_index_reduces_page_reads() {
    let path = std::env::temp_dir().join(format!(
        "dreamizdb-test-{}.db",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let table = sample_table(20_000);
    let mut disk = PersistentTable::create_from_table(&path, &table).unwrap();

    let scan = benchmark_persistent_scan(&mut disk, "BD", 1).unwrap();
    let indexed = benchmark_persistent_index(&mut disk, "BD", 1).unwrap();

    assert_eq!(scan.rows, indexed.rows);
    assert!(scan.pages_read > indexed.pages_read);
    assert!(scan.bytes_read > indexed.bytes_read);

    drop(disk);
    let mut reopened = PersistentTable::open(&path).unwrap();
    assert_eq!(reopened.indexed_lookup("BD").unwrap().len(), scan.rows);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
