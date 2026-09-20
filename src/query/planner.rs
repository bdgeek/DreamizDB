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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStrategy {
    SequentialScan,
    IndexedLookup,
}

impl PlanStrategy {
    pub fn name(self) -> &'static str {
        match self {
            Self::SequentialScan => "Sequential Scan",
            Self::IndexedLookup => "Indexed Lookup",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanEstimate {
    pub strategy: PlanStrategy,
    pub estimated_rows: usize,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplainPlan {
    pub table: String,
    pub predicate: Option<PredicateExpression>,
    pub candidates: Vec<PlanEstimate>,
    pub selected_strategy: PlanStrategy,
    pub selected_plan: QueryPlan,
    pub selected_cost: f64,
}

impl ExplainPlan {
    pub fn selected_estimate(&self) -> Option<&PlanEstimate> {
        self.candidates
            .iter()
            .find(|candidate| candidate.strategy == self.selected_strategy)
    }

    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("QUERY PLAN\n");
        output.push_str("--------------------------------\n");
        output.push_str(&format!("Table: {}\n", self.table));

        if self.predicate.is_some() {
            output.push_str("Predicate: present\n");
        } else {
            output.push_str("Predicate: none\n");
        }

        output.push_str("\nCandidate Plans:\n");

        for candidate in &self.candidates {
            output.push_str(&format!("  {}\n", candidate.strategy.name()));
            output.push_str(&format!(
                "    estimated rows: {}\n",
                candidate.estimated_rows
            ));
            output.push_str(&format!(
                "    estimated cost: {:.2}\n",
                candidate.estimated_cost
            ));
        }

        output.push_str("\nSelected:\n");
        output.push_str(&format!("  {}\n", self.selected_strategy.name()));

        output.push_str(&format!("  estimated cost: {:.2}\n", self.selected_cost));

        if self.candidates.len() > 1 {
            output.push_str("\nReason:\n");
            output.push_str("  lowest estimated cost\n");
        }

        output
    }
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
    Ok(explain_query_with_stats(query, stats)?.selected_plan)
}

/// Cost-based planner using statistics directly from a persistent table.
pub fn plan_query_for_table(
    query: &SelectQuery,
    table: &PersistentTable,
) -> anyhow::Result<QueryPlan> {
    Ok(explain_query_for_table(query, table)?.selected_plan)
}

/// Build an explanation using supplied statistics.
///
/// This does not execute the query.
pub fn explain_query_with_stats(
    query: &SelectQuery,
    stats: QueryStatistics,
) -> anyhow::Result<ExplainPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    let sequential = PlanEstimate {
        strategy: PlanStrategy::SequentialScan,
        estimated_rows: stats.total_rows,
        estimated_cost: stats.sequential_cost(),
    };

    let indexable_predicate = find_indexable_country_equality(&query.predicate);

    let mut candidates = vec![sequential];

    if indexable_predicate.is_some() && stats.indexed_rows > 0 {
        candidates.push(PlanEstimate {
            strategy: PlanStrategy::IndexedLookup,
            estimated_rows: stats.matching_rows,
            estimated_cost: stats.indexed_cost(),
        });
    }

    let selected_strategy = if stats.should_use_index() && indexable_predicate.is_some() {
        PlanStrategy::IndexedLookup
    } else {
        PlanStrategy::SequentialScan
    };

    let selected_plan = match selected_strategy {
        PlanStrategy::SequentialScan => sequential_plan(query),
        PlanStrategy::IndexedLookup => {
            indexed_plan(query, indexable_predicate.expect("indexable predicate"))
        }
    };

    let selected_cost = candidates
        .iter()
        .find(|candidate| candidate.strategy == selected_strategy)
        .map(|candidate| candidate.estimated_cost)
        .unwrap_or_else(|| stats.sequential_cost());

    Ok(ExplainPlan {
        table: query.table.clone(),
        predicate: query.predicate.clone(),
        candidates,
        selected_strategy,
        selected_plan,
        selected_cost,
    })
}

/// Build an explanation using statistics directly from persistent storage.
///
/// This does not execute the query.
pub fn explain_query_for_table(
    query: &SelectQuery,
    table: &PersistentTable,
) -> anyhow::Result<ExplainPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    let predicate = find_indexable_country_equality(&query.predicate);

    let stats = match predicate {
        Some(predicate) => table.query_statistics_for_country(&predicate.value),
        None => QueryStatistics::new(table.page_count() as usize, table.indexed_row_count(), 0),
    };

    explain_query_with_stats(query, stats)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parse_select;

    #[test]
    fn explain_small_match_set_contains_both_candidates() {
        let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

        let stats = QueryStatistics::new(1000, 1000, 10);

        let explain = explain_query_with_stats(&query, stats).unwrap();

        assert_eq!(explain.candidates.len(), 2);
        assert_eq!(explain.selected_strategy, PlanStrategy::IndexedLookup);
        assert_eq!(explain.selected_cost, stats.indexed_cost());
    }

    #[test]
    fn explain_large_match_set_selects_sequential_scan() {
        let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

        let stats = QueryStatistics::new(1000, 1000, 990);

        let explain = explain_query_with_stats(&query, stats).unwrap();

        assert_eq!(explain.selected_strategy, PlanStrategy::SequentialScan);
    }

    #[test]
    fn explain_non_indexable_predicate_has_one_candidate() {
        let query = parse_select("SELECT * FROM users WHERE value > 100").unwrap();

        let stats = QueryStatistics::new(1000, 1000, 100);

        let explain = explain_query_with_stats(&query, stats).unwrap();

        assert_eq!(explain.candidates.len(), 1);
        assert_eq!(explain.selected_strategy, PlanStrategy::SequentialScan);
    }

    #[test]
    fn explain_format_contains_selected_strategy() {
        let query = parse_select("SELECT * FROM users WHERE country = 'BD'").unwrap();

        let stats = QueryStatistics::new(1000, 1000, 10);

        let explain = explain_query_with_stats(&query, stats).unwrap();

        let text = explain.format_text();

        assert!(text.contains("QUERY PLAN"));
        assert!(text.contains("Sequential Scan"));
        assert!(text.contains("Indexed Lookup"));
        assert!(text.contains("Selected:"));
    }
}
