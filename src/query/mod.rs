mod executor;
mod metrics;
mod parser;
mod planner;
mod types;

pub use executor::{
    execute_query, execute_query_with_metrics, QueryExecution, QueryResult, QueryRow,
};

pub use metrics::{QueryExecutionMetrics, QueryMetrics};

pub use parser::parse_select;

pub use planner::{
    explain_query_for_table, explain_query_with_stats, plan_query, plan_query_for_table,
    plan_query_with_stats, ExplainPlan, PlanEstimate, PlanStrategy, QueryPlan,
};

pub use types::{ComparisonOperator, PredicateExpression, PredicatePlan, SelectQuery};

pub use crate::statistics::QueryStatistics;
