use dreamizdb::query::{parse_select, plan_query_for_table, QueryPlan};
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

    for id in 1..=1000 {
        table.insert(Record {
            id,
            country: if id <= 10 { "BD".into() } else { "IN".into() },
            value: id as f64,
        });
    }

    table
}

#[test]
fn persistent_statistics_drive_index_choice_for_small_match_set() {
    let (data_path, index_path) = test_paths("cost-planner-small-match");

    let table = sample_table();

    let persistent = PersistentTable::create_from_table(&data_path, &table).unwrap();

    assert_eq!(persistent.page_count(), 1000);
    assert_eq!(persistent.indexed_row_count(), 1000);
    assert_eq!(persistent.country_index_match_count("BD"), 10);

    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query_for_table(&query, &persistent).expect("query should plan");

    assert!(matches!(plan, QueryPlan::IndexedLookup { .. }));

    cleanup(&data_path, &index_path);
}

#[test]
fn persistent_statistics_drive_scan_choice_for_large_match_set() {
    let (data_path, index_path) = test_paths("cost-planner-large-match");

    let table = sample_table();

    let persistent = PersistentTable::create_from_table(&data_path, &table).unwrap();

    assert_eq!(persistent.page_count(), 1000);
    assert_eq!(persistent.indexed_row_count(), 1000);
    assert_eq!(persistent.country_index_match_count("IN"), 990);

    let query =
        parse_select("SELECT * FROM users WHERE country = 'IN'").expect("query should parse");

    let plan = plan_query_for_table(&query, &persistent).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));

    cleanup(&data_path, &index_path);
}
