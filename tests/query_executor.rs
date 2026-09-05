use dreamizdb::query::{execute_query, parse_select, plan_query, QueryResult};
use dreamizdb::storage::persistence::PersistentTable;
use dreamizdb::storage::{Record, Table};

fn create_test_table() -> (PersistentTable, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("temporary directory should be created");
    let path = temp_dir.path().join("users.db");

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

    table.insert(Record {
        id: 4,
        country: "IN".into(),
        value: 400.0,
    });

    let persistent =
        PersistentTable::create_from_table(&path, &table).expect("table should be created");

    (persistent, temp_dir)
}

fn execute(sql: &str, use_index: bool) -> QueryResult {
    let (mut table, _temp_dir) = create_test_table();

    let query = parse_select(sql).expect("query should parse");
    let plan = plan_query(&query, use_index).expect("query should plan");

    execute_query(&mut table, &plan).expect("query should execute")
}

#[test]
fn executes_select_without_predicate() {
    let result = execute("SELECT * FROM users", true);

    assert_eq!(result.columns, vec!["*"]);
    assert_eq!(result.rows.len(), 4);

    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["2,US,200"]);
    assert_eq!(result.rows[2].values, vec!["3,BD,300"]);
    assert_eq!(result.rows[3].values, vec!["4,IN,400"]);
}

#[test]
fn executes_country_predicate_with_sequential_scan() {
    let result = execute("SELECT * FROM users WHERE country = 'BD'", false);

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["3,BD,300"]);
}

#[test]
fn executes_country_predicate_with_index() {
    let result = execute("SELECT * FROM users WHERE country = 'BD'", true);

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["3,BD,300"]);
}

#[test]
fn indexed_and_sequential_execution_return_same_rows() {
    let indexed = execute("SELECT * FROM users WHERE country = 'BD'", true);
    let sequential = execute("SELECT * FROM users WHERE country = 'BD'", false);

    assert_eq!(indexed, sequential);
}

#[test]
fn executes_not_equal_predicate() {
    let result = execute("SELECT * FROM users WHERE country != 'BD'", false);

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["2,US,200"]);
    assert_eq!(result.rows[1].values, vec!["4,IN,400"]);
}

#[test]
fn executes_less_than_predicate() {
    let result = execute("SELECT * FROM users WHERE value < 200", false);

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
}

#[test]
fn executes_less_than_or_equal_predicate() {
    let result = execute("SELECT * FROM users WHERE value <= 200", false);

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["2,US,200"]);
}

#[test]
fn executes_greater_than_predicate() {
    let result = execute("SELECT * FROM users WHERE value > 200", false);

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["3,BD,300"]);
    assert_eq!(result.rows[1].values, vec!["4,IN,400"]);
}

#[test]
fn executes_greater_than_or_equal_predicate() {
    let result = execute("SELECT * FROM users WHERE value >= 200", false);

    assert_eq!(result.rows.len(), 3);
    assert_eq!(result.rows[0].values, vec!["2,US,200"]);
    assert_eq!(result.rows[1].values, vec!["3,BD,300"]);
    assert_eq!(result.rows[2].values, vec!["4,IN,400"]);
}

#[test]
fn executes_and_predicate() {
    let result = execute(
        "SELECT * FROM users WHERE country = 'BD' AND value > 100",
        false,
    );

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].values, vec!["3,BD,300"]);
}

#[test]
fn executes_or_predicate() {
    let result = execute(
        "SELECT * FROM users WHERE country = 'BD' OR country = 'US'",
        false,
    );

    assert_eq!(result.rows.len(), 3);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["2,US,200"]);
    assert_eq!(result.rows[2].values, vec!["3,BD,300"]);
}

#[test]
fn executes_nested_and_or_predicate() {
    let result = execute(
        "SELECT * FROM users WHERE country = 'BD' AND (value >= 100 OR value = 400)",
        false,
    );

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["1,BD,100"]);
    assert_eq!(result.rows[1].values, vec!["3,BD,300"]);
}

#[test]
fn executes_numeric_range_with_and() {
    let result = execute(
        "SELECT * FROM users WHERE value >= 200 AND value <= 300",
        false,
    );

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values, vec!["2,US,200"]);
    assert_eq!(result.rows[1].values, vec!["3,BD,300"]);
}

#[test]
fn projection_returns_only_requested_columns() {
    let result = execute("SELECT id, country FROM users WHERE country = 'BD'", true);

    assert_eq!(result.columns, vec!["id", "country"]);
    assert_eq!(result.rows.len(), 2);

    assert_eq!(result.rows[0].values, vec!["1", "BD"]);
    assert_eq!(result.rows[1].values, vec!["3", "BD"]);
}
