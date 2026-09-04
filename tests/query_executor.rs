use dreamizdb::query::{execute_query, parse_select, plan_query};
use dreamizdb::storage::persistence::PersistentTable;
use dreamizdb::storage::{Record, Table};

fn test_table() -> (PersistentTable, std::path::PathBuf) {
    let unique_id = format!("{}-{:?}", std::process::id(), std::thread::current().id())
        .replace(['(', ')', ' '], "_");

    let path = std::env::temp_dir().join(format!("dreamizdb-query-executor-{unique_id}.db"));

    let mut table = Table::new();

    table.insert(Record {
        id: 1,
        country: "BD".into(),
        value: 100.0,
    });

    table.insert(Record {
        id: 2,
        country: "US".into(),
        value: 200.0,
    });

    table.insert(Record {
        id: 3,
        country: "BD".into(),
        value: 300.0,
    });

    let persistent =
        PersistentTable::create_from_table(&path, &table).expect("table should be created");

    (persistent, path)
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("idx"));
}

#[test]
fn executes_select_without_predicate() {
    let (mut table, path) = test_table();

    let query = parse_select("SELECT * FROM users").expect("query should parse");
    let plan = plan_query(&query, false).expect("query should plan");

    let result = execute_query(&mut table, &plan).expect("query should execute");

    assert_eq!(result.columns, vec!["*"]);
    assert_eq!(result.rows.len(), 3);

    cleanup(&path);
}

#[test]
fn executes_country_predicate_with_sequential_scan() {
    let (mut table, path) = test_table();

    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query(&query, false).expect("query should plan");

    let result = execute_query(&mut table, &plan).expect("query should execute");

    assert_eq!(result.rows.len(), 2);

    cleanup(&path);
}

#[test]
fn executes_country_predicate_with_index() {
    let (mut table, path) = test_table();

    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    let result = execute_query(&mut table, &plan).expect("query should execute");

    assert_eq!(result.rows.len(), 2);

    cleanup(&path);
}

#[test]
fn indexed_and_sequential_execution_return_same_rows() {
    let (mut table, path) = test_table();

    let query = parse_select("SELECT id, country, value FROM users WHERE country = 'BD'")
        .expect("query should parse");

    let sequential_plan = plan_query(&query, false).expect("query should plan");
    let indexed_plan = plan_query(&query, true).expect("query should plan");

    let sequential_result =
        execute_query(&mut table, &sequential_plan).expect("sequential query should execute");

    table.clear_cache();
    table.reset_metrics();

    let indexed_result =
        execute_query(&mut table, &indexed_plan).expect("indexed query should execute");

    assert_eq!(sequential_result, indexed_result);

    cleanup(&path);
}

#[test]
fn projection_returns_only_requested_columns() {
    let (mut table, path) = test_table();

    let query = parse_select("SELECT id, country FROM users WHERE country = 'BD'")
        .expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    let result = execute_query(&mut table, &plan).expect("query should execute");

    assert_eq!(result.columns, vec!["id", "country"]);
    assert_eq!(result.rows.len(), 2);

    assert_eq!(result.rows[0].values.len(), 2);
    assert_eq!(result.rows[1].values.len(), 2);

    cleanup(&path);
}
