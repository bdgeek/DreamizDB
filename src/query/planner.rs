use super::types::{ComparisonOperator, PredicateExpression, PredicatePlan, SelectQuery};
use crate::statistics::QueryStatistics;
use crate::storage::persistence::PersistentTable;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryPlan {
    SequentialScan {
        table: String,
        columns: Vec<String>,
        predicate: Option<PredicateExpression>,
    },
    IndexedLookup {
        table: String,
        columns: Vec<String>,
        column: String,
        value: String,
        predicate: Option<PredicateExpression>,
    },
}

/// Compatibility planner using only index availability.
pub fn plan_query(query: &SelectQuery, has_country_index: bool) -> anyhow::Result<QueryPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    if let Some(predicate) = find_indexable_country_equality(&query.predicate) {
        if has_country_index {
            return Ok(indexed_plan(query, predicate));
        }
    }

    Ok(sequential_plan(query))
}

/// Cost-based planner using supplied persistent statistics.
pub fn plan_query_with_stats(
    query: &SelectQuery,
    stats: QueryStatistics,
) -> anyhow::Result<QueryPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    if let Some(predicate) = find_indexable_country_equality(&query.predicate) {
        if stats.should_use_index() {
            return Ok(indexed_plan(query, predicate));
        }
    }

    Ok(sequential_plan(query))
}

/// Cost-based planner using statistics directly from a persistent table.
pub fn plan_query_for_table(
    query: &SelectQuery,
    table: &PersistentTable,
) -> anyhow::Result<QueryPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    let predicate = match find_indexable_country_equality(&query.predicate) {
        Some(predicate) => predicate,
        None => return Ok(sequential_plan(query)),
    };

    let stats = table.query_statistics_for_country(&predicate.value);

    if stats.should_use_index() {
        Ok(indexed_plan(query, predicate))
    } else {
        Ok(sequential_plan(query))
    }
}

fn indexed_plan(query: &SelectQuery, predicate: &PredicatePlan) -> QueryPlan {
    QueryPlan::IndexedLookup {
        table: query.table.clone(),
        columns: query.columns.clone(),
        column: predicate.column.clone(),
        value: predicate.value.clone(),
        predicate: query.predicate.clone(),
    }
}

fn sequential_plan(query: &SelectQuery) -> QueryPlan {
    QueryPlan::SequentialScan {
        table: query.table.clone(),
        columns: query.columns.clone(),
        predicate: query.predicate.clone(),
    }
}

fn find_indexable_country_equality(
    predicate: &Option<PredicateExpression>,
) -> Option<&PredicatePlan> {
    match predicate {
        Some(PredicateExpression::Comparison(plan))
            if plan.column.eq_ignore_ascii_case("country")
                && plan.operator == ComparisonOperator::Equal =>
        {
            Some(plan)
        }

        Some(PredicateExpression::Comparison(_)) => None,

        Some(PredicateExpression::And(left, right)) => {
            find_indexable_country_equality_from_expression(left)
                .or_else(|| find_indexable_country_equality_from_expression(right))
        }

        Some(PredicateExpression::Or(_, _)) | None => None,
    }
}

fn find_indexable_country_equality_from_expression(
    predicate: &PredicateExpression,
) -> Option<&PredicatePlan> {
    match predicate {
        PredicateExpression::Comparison(plan)
            if plan.column.eq_ignore_ascii_case("country")
                && plan.operator == ComparisonOperator::Equal =>
        {
            Some(plan)
        }

        PredicateExpression::Comparison(_) => None,

        PredicateExpression::And(left, right) => {
            find_indexable_country_equality_from_expression(left)
                .or_else(|| find_indexable_country_equality_from_expression(right))
        }

        PredicateExpression::Or(_, _) => None,
    }
}
