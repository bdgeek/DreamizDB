#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectQuery {
    pub columns: Vec<String>,
    pub table: String,
    pub predicate: Option<Predicate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Predicate {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}
