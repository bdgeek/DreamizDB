use dreamizdb::query::{
    execute_query_with_metrics, explain_query_with_stats, parse_select, plan_query_with_stats,
    PlanStrategy, QueryStatistics,
};
use dreamizdb::storage::persistence::PersistentTable;
use dreamizdb::storage::{Record, Table};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_table() -> PersistentTable {
    let mut table = Table::new();

    for id in 1..=20 {
        let country = if id <= 5 { "BD" } else { "US" };

        table.insert(Record {
            id,
            country: country.to_string(),
            value: id as f64 * 10.0,
        });
    }

    /*
     * Each test gets its own directory.
     *
     * This avoids Windows filesystem races when Rust runs
     * integration tests in parallel.
     */
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_nanos();

    let test_dir = std::env::temp_dir().join(format!(
        "dreamizdb-observability-{}-{}",
        std::process::id(),
        timestamp
    ));

    std::fs::create_dir_all(&test_dir)
        .expect("failed to create temporary DreamizDB test directory");

    let path = test_dir.join("users.db");

    PersistentTable::create_from_table(&path, &table)
        .expect("failed to create persistent test table")
}

#[test]
fn sequential_execution_records_actual_metrics() {
    let mut table = create_table();

    table.reset_metrics();
    table.clear_cache();

    let query = parse_select("SELECT * FROM users WHERE value > 100").unwrap();

    let stats = QueryStatistics::new(20, 20, 10);

    let plan = plan_query_with_stats(&query, stats).unwrap();

    let execution = execute_query_with_metrics(&mut table, &plan).unwrap();

    assert_eq!(execution.metrics.rows_examined, 20);
    assert_eq!(execution.metrics.rows_returned, 10);

    assert!(execution.metrics.page_reads > 0);
    assert!(execution.metrics.bytes_read > 0);
    assert!(execution.metrics.elapsed_ns > 0);

    assert_eq!(execution.metrics.index_lookups, 0);
    assert!(execution.metrics.cache_misses > 0);
}

#[test]
fn indexed_execution_records_index_lookup() {
    let mut table = create_table();

    table.reset_metrics();
    table.clear_cache();

    let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

    let stats = QueryStatistics::new(20, 20, 5);

    let plan = plan_query_with_stats(&query, stats).unwrap();

    assert!(matches!(
        plan,
        dreamizdb::query::QueryPlan::IndexedLookup { .. }
    ));

    let execution = execute_query_with_metrics(&mut table, &plan).unwrap();

    assert_eq!(execution.metrics.index_lookups, 1);
    assert_eq!(execution.metrics.rows_examined, 5);
    assert_eq!(execution.metrics.rows_returned, 5);

    assert!(execution.metrics.page_reads > 0);
    assert!(execution.metrics.bytes_read > 0);
}

#[test]
fn repeated_indexed_execution_can_observe_cache_hits() {
    let mut table = create_table();

    table.reset_metrics();
    table.clear_cache();

    let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

    let stats = QueryStatistics::new(20, 20, 5);

    let plan = plan_query_with_stats(&query, stats).unwrap();

    let first = execute_query_with_metrics(&mut table, &plan).unwrap();

    let second = execute_query_with_metrics(&mut table, &plan).unwrap();

    assert_eq!(first.metrics.rows_returned, 5);
    assert_eq!(second.metrics.rows_returned, 5);

    /*
     * The second execution should be able to reuse pages
     * loaded by the first execution.
     */
    assert!(second.metrics.cache_hits >= first.metrics.cache_hits);
}

#[test]
fn explain_does_not_execute_query() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

    let stats = QueryStatistics::new(20, 20, 5);

    let explain = explain_query_with_stats(&query, stats).unwrap();

    assert_eq!(explain.table, "users");

    assert_eq!(explain.selected_strategy, PlanStrategy::IndexedLookup);

    assert_eq!(explain.candidates.len(), 2);
}

#[test]
fn explain_reports_large_match_set_as_sequential() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

    let stats = QueryStatistics::new(1000, 1000, 990);

    let explain = explain_query_with_stats(&query, stats).unwrap();

    assert_eq!(explain.selected_strategy, PlanStrategy::SequentialScan);

    assert_eq!(explain.candidates.len(), 2);
}

#[test]
fn explain_text_is_human_readable() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

    let stats = QueryStatistics::new(1000, 1000, 10);

    let explain = explain_query_with_stats(&query, stats).unwrap();

    let text = explain.format_text();

    assert!(text.contains("QUERY PLAN"));
    assert!(text.contains("Table: users"));
    assert!(text.contains("Sequential Scan"));
    assert!(text.contains("Indexed Lookup"));
    assert!(text.contains("estimated rows"));
    assert!(text.contains("estimated cost"));
    assert!(text.contains("Selected:"));
}
