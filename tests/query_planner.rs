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
            predicate: None,
        }
    );
}

#[test]
fn chooses_scan_without_country_index() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    let plan = plan_query(&query, false).expect("query should plan");

    assert!(matches!(
        plan,
        QueryPlan::SequentialScan {
            predicate: Some(_),
            ..
        }
    ));
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

#[test]
fn chooses_index_for_country_equality_with_additional_and_predicate() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD' AND value > 100")
        .expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    match plan {
        QueryPlan::IndexedLookup {
            table,
            columns,
            column,
            value,
            predicate,
        } => {
            assert_eq!(table, "users");
            assert_eq!(columns, vec!["*"]);
            assert_eq!(column, "country");
            assert_eq!(value, "BD");
            assert!(predicate.is_some());
        }
        other => panic!("expected IndexedLookup, got {other:?}"),
    }
}

#[test]
fn chooses_scan_for_country_equality_with_or_predicate() {
    let query = parse_select("SELECT * FROM users WHERE country = 'BD' OR value > 100")
        .expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    assert!(matches!(
        plan,
        QueryPlan::SequentialScan {
            predicate: Some(_),
            ..
        }
    ));
}

#[test]
fn chooses_index_when_country_predicate_is_on_right_side_of_and() {
    let query = parse_select("SELECT * FROM users WHERE value > 100 AND country = 'BD'")
        .expect("query should parse");

    let plan = plan_query(&query, true).expect("query should plan");

    assert!(matches!(
        plan,
        QueryPlan::IndexedLookup {
            table,
            columns,
            column,
            value,
            predicate: Some(_),
        } if table == "users"
            && columns == vec!["*"]
            && column == "country"
            && value == "BD"
    ));
}
