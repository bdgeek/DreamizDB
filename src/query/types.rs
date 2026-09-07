use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredicatePlan {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateExpression {
    Comparison(PredicatePlan),
    And(Box<PredicateExpression>, Box<PredicateExpression>),
    Or(Box<PredicateExpression>, Box<PredicateExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub predicate: Option<PredicateExpression>,
}
