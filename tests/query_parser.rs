use dreamizdb::query::{parse_select, ComparisonOperator};

#[test]
fn parses_select_with_country_predicate() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD';").expect("query should parse");

    assert_eq!(query.columns, vec!["*"]);
    assert_eq!(query.table, "users");

    let predicate = query.predicate.expect("predicate should exist");

    assert_eq!(predicate.column, "country");
    assert_eq!(predicate.operator, ComparisonOperator::Equal);
    assert_eq!(predicate.value, "BD");
}

#[test]
fn parses_multiple_columns() {
    let query = parse_select("SELECT id, country, value FROM users WHERE country = 'BD'")
        .expect("query should parse");

    assert_eq!(query.columns, vec!["id", "country", "value"]);
    assert_eq!(query.table, "users");
}

#[test]
fn parses_select_without_predicate() {
    let query = parse_select("SELECT * FROM users").expect("query should parse");

    assert_eq!(query.columns, vec!["*"]);
    assert_eq!(query.table, "users");
    assert!(query.predicate.is_none());
}

#[test]
fn rejects_non_select_query() {
    assert!(parse_select("INSERT INTO users VALUES (1)").is_err());
}

#[test]
fn rejects_missing_from() {
    assert!(parse_select("SELECT * users").is_err());
}
