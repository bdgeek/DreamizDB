use dreamizdb::query::{parse_select, plan_query_with_stats, QueryPlan, QueryStatistics};

#[test]
fn chooses_index_when_match_set_is_small() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let stats = QueryStatistics::new(1000, 1000, 10);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::IndexedLookup { .. }));
}

#[test]
fn chooses_scan_when_match_set_is_large() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    // indexed cost = 1 + 1000 = 1001
    // sequential cost = 1000
    let stats = QueryStatistics::new(1000, 1000, 1000);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}

#[test]
fn chooses_scan_when_no_indexed_rows_exist() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let stats = QueryStatistics::new(1000, 0, 10);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}

#[test]
fn chooses_scan_for_non_country_predicate() {
    let query = parse_select("SELECT * FROM users WHERE value > 50").expect("query should parse");

    let stats = QueryStatistics::new(1000, 1000, 10);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}

#[test]
fn chooses_scan_for_or_predicate() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD' OR value > 50")
        .expect("query should parse");

    let stats = QueryStatistics::new(1000, 1000, 10);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}

#[test]
fn can_use_index_for_country_predicate_inside_and() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD' AND value > 50")
        .expect("query should parse");

    let stats = QueryStatistics::new(1000, 1000, 10);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::IndexedLookup { .. }));
}

#[test]
fn cost_boundary_prefers_scan_when_costs_are_equal() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    // indexed cost = 1 + 999 = 1000
    // sequential cost = 1000
    let stats = QueryStatistics::new(1000, 1000, 999);

    let plan = plan_query_with_stats(&query, stats).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}
