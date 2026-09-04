use dreamizdb::query::{parse_select, plan_query, QueryPlan};

#[test]
fn chooses_index_for_country_equality_when_index_exists() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    assert_eq!(
        plan,
        QueryPlan::IndexedLookup {
            table: "users".into(),
            columns: vec!["*".into()],
            column: "country".into(),
            value: "BD".into(),
        }
    );
}

#[test]
fn chooses_scan_without_country_index() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query(&query, false).expect("query should plan");

    assert!(matches!(plan, QueryPlan::SequentialScan { .. }));
}

#[test]
fn chooses_scan_for_query_without_predicate() {
    let query = parse_select("SELECT * FROM users").expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    assert!(matches!(
        plan,
        QueryPlan::SequentialScan {
            predicate: None,
            ..
        }
    ));
}

#[test]
fn chooses_scan_for_non_country_predicate() {
    let query =
        parse_select("SELECT * FROM users WHERE name = 'Alice'").expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    assert!(matches!(
        plan,
        QueryPlan::SequentialScan {
            predicate: Some(_),
            ..
        }
    ));
}
