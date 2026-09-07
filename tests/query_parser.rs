use dreamizdb::query::{parse_select, ComparisonOperator, PredicateExpression, PredicatePlan};

#[test]
fn parses_select_without_predicate() {
    let query = parse_select("SELECT * FROM users").expect("query should parse");

    assert_eq!(query.columns, vec!["*"]);
    assert_eq!(query.table, "users");
    assert!(query.predicate.is_none());
}

#[test]
fn parses_select_with_country_predicate() {
    let query =
        parse_select("SELECT * FROM users WHERE country = 'BD'").expect("query should parse");

    assert_eq!(query.columns, vec!["*"]);
    assert_eq!(query.table, "users");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "country".into(),
            operator: ComparisonOperator::Equal,
            value: "BD".into(),
        }))
    );
}

#[test]
fn parses_multiple_columns() {
    let query = parse_select("SELECT id, country, value FROM users").expect("query should parse");

    assert_eq!(query.columns, vec!["id", "country", "value"]);
    assert_eq!(query.table, "users");
    assert!(query.predicate.is_none());
}

#[test]
fn parses_and_predicate() {
    let query = parse_select("SELECT id FROM users WHERE country = 'BD' AND value > 100")
        .expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::And(
            Box::new(PredicateExpression::Comparison(PredicatePlan {
                column: "country".into(),
                operator: ComparisonOperator::Equal,
                value: "BD".into(),
            })),
            Box::new(PredicateExpression::Comparison(PredicatePlan {
                column: "value".into(),
                operator: ComparisonOperator::GreaterThan,
                value: "100".into(),
            })),
        ))
    );
}

#[test]
fn parses_or_predicate() {
    let query = parse_select("SELECT id FROM users WHERE country = 'BD' OR country = 'US'")
        .expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Or(
            Box::new(PredicateExpression::Comparison(PredicatePlan {
                column: "country".into(),
                operator: ComparisonOperator::Equal,
                value: "BD".into(),
            })),
            Box::new(PredicateExpression::Comparison(PredicatePlan {
                column: "country".into(),
                operator: ComparisonOperator::Equal,
                value: "US".into(),
            })),
        ))
    );
}

#[test]
fn parses_nested_and_or_predicate() {
    let query =
        parse_select("SELECT id FROM users WHERE country = 'BD' AND value > 100 OR country = 'US'")
            .expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Or(
            Box::new(PredicateExpression::And(
                Box::new(PredicateExpression::Comparison(PredicatePlan {
                    column: "country".into(),
                    operator: ComparisonOperator::Equal,
                    value: "BD".into(),
                })),
                Box::new(PredicateExpression::Comparison(PredicatePlan {
                    column: "value".into(),
                    operator: ComparisonOperator::GreaterThan,
                    value: "100".into(),
                })),
            )),
            Box::new(PredicateExpression::Comparison(PredicatePlan {
                column: "country".into(),
                operator: ComparisonOperator::Equal,
                value: "US".into(),
            })),
        ))
    );
}

#[test]
fn parses_not_equal_operator() {
    let query =
        parse_select("SELECT id FROM users WHERE country != 'BD'").expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "country".into(),
            operator: ComparisonOperator::NotEqual,
            value: "BD".into(),
        }))
    );
}

#[test]
fn parses_less_than_operator() {
    let query = parse_select("SELECT id FROM users WHERE value < 200").expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "value".into(),
            operator: ComparisonOperator::LessThan,
            value: "200".into(),
        }))
    );
}

#[test]
fn parses_less_than_or_equal_operator() {
    let query =
        parse_select("SELECT id FROM users WHERE value <= 200").expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "value".into(),
            operator: ComparisonOperator::LessThanOrEqual,
            value: "200".into(),
        }))
    );
}

#[test]
fn parses_greater_than_operator() {
    let query = parse_select("SELECT id FROM users WHERE value > 100").expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "value".into(),
            operator: ComparisonOperator::GreaterThan,
            value: "100".into(),
        }))
    );
}

#[test]
fn parses_greater_than_or_equal_operator() {
    let query =
        parse_select("SELECT id FROM users WHERE value >= 100").expect("query should parse");

    assert_eq!(
        query.predicate,
        Some(PredicateExpression::Comparison(PredicatePlan {
            column: "value".into(),
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: "100".into(),
        }))
    );
}

#[test]
fn rejects_non_select_query() {
    let result = parse_select("INSERT INTO users VALUES (1, 'BD', 100)");

    assert!(result.is_err());
}

#[test]
fn rejects_missing_from() {
    let result = parse_select("SELECT id users");

    assert!(result.is_err());
}
