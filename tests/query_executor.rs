use dreamizdb::query::executor::execute_query;
use dreamizdb::query::{parse_select, plan_query};
use dreamizdb::storage::persistence::PersistentTable;
use dreamizdb::storage::{Record, Table};
use std::fs;

fn persistent_table() -> (PersistentTable, std::path::PathBuf) {
    let unique_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let path = std::env::temp_dir().join(format!("dreamizdb-query-executor-{unique_id}.db"));

    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(path.with_extension("idx"));

    let mut table = Table::new();

    table.insert(Record {
        id: 1,
        country: "BD".into(),
        value: 10.0,
    });

    table.insert(Record {
        id: 2,
        country: "IN".into(),
        value: 20.0,
    });

    table.insert(Record {
        id: 3,
        country: "BD".into(),
        value: 30.0,
    });

    let persistent = PersistentTable::create_from_table(&path, &table).unwrap();

    (persistent, path)
}

fn cleanup(path: &std::path::Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("idx"));
}

#[test]
fn executes_select_without_predicate() {
    let (mut table, path) = persistent_table();

    let query = parse_select("SELECT id, country, value FROM users").unwrap();
    let plan = plan_query(&query, false).unwrap();

    let result = execute_query(&mut table, &plan).unwrap();

    assert_eq!(result.columns, vec!["id", "country", "value"]);
    assert_eq!(result.rows.len(), 3);

    assert_eq!(result.rows[0].values, vec!["1", "BD", "10"]);

    cleanup(&path);
}

#[test]
fn executes_country_predicate_with_sequential_scan() {
    let (mut table, path) = persistent_table();

    let query = parse_select("SELECT id, country FROM users WHERE country = 'BD'").unwrap();

    let plan = plan_query(&query, false).unwrap();

    let result = execute_query(&mut table, &plan).unwrap();

    assert_eq!(result.columns, vec!["id", "country"]);
    assert_eq!(result.rows.len(), 2);

    assert_eq!(result.rows[0].values, vec!["1", "BD"]);
    assert_eq!(result.rows[1].values, vec!["3", "BD"]);

    cleanup(&path);
}

#[test]
fn executes_country_predicate_with_index() {
    let (mut table, path) = persistent_table();

    let query = parse_select("SELECT id, country, value FROM users WHERE country = 'BD'").unwrap();

    let plan = plan_query(&query, true).unwrap();

    let result = execute_query(&mut table, &plan).unwrap();

    assert_eq!(result.columns, vec!["id", "country", "value"]);
    assert_eq!(result.rows.len(), 2);

    assert_eq!(result.rows[0].values, vec!["1", "BD", "10"]);

    assert_eq!(result.rows[1].values, vec!["3", "BD", "30"]);

    cleanup(&path);
}

#[test]
fn indexed_and_sequential_execution_return_same_rows() {
    let (mut table, path) = persistent_table();

    let query = parse_select("SELECT id, country, value FROM users WHERE country = 'BD'").unwrap();

    let scan_plan = plan_query(&query, false).unwrap();
    let index_plan = plan_query(&query, true).unwrap();

    let scan_result = execute_query(&mut table, &scan_plan).unwrap();
    let index_result = execute_query(&mut table, &index_plan).unwrap();

    assert_eq!(scan_result, index_result);

    cleanup(&path);
}

#[test]
fn projection_returns_only_requested_columns() {
    let (mut table, path) = persistent_table();

    let query = parse_select("SELECT country FROM users").unwrap();
    let plan = plan_query(&query, false).unwrap();

    let result = execute_query(&mut table, &plan).unwrap();

    assert_eq!(result.columns, vec!["country"]);
    assert_eq!(result.rows.len(), 3);

    assert_eq!(result.rows[0].values, vec!["BD"]);
    assert_eq!(result.rows[1].values, vec!["IN"]);
    assert_eq!(result.rows[2].values, vec!["BD"]);

    cleanup(&path);
}
