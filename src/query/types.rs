use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ComparisonOperator {
    pub fn parse(operator: &str) -> Result<Self> {
        match operator {
            "=" => Ok(Self::Equal),
            "!=" | "<>" => Ok(Self::NotEqual),
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEqual),
            ">" => Ok(Self::GreaterThan),
            ">=" => Ok(Self::GreaterThanOrEqual),
            _ => Err(anyhow!("unsupported comparison operator: {operator}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicatePlan {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateExpression {
    Comparison(PredicatePlan),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

/// Backward-compatible alias.
pub type Predicate = PredicateExpression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectQuery {
    pub columns: Vec<String>,
    pub table: String,
    pub predicate: Option<PredicateExpression>,
}
