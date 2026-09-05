use super::types::{ComparisonOperator, PredicateExpression, SelectQuery};

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub fn plan_query(query: &SelectQuery, has_country_index: bool) -> anyhow::Result<QueryPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    if !has_country_index {
        return Ok(QueryPlan::SequentialScan {
            table: query.table.clone(),
            columns: query.columns.clone(),
            predicate: query.predicate.clone(),
        });
    }

    if let Some((column, value, residual)) = find_country_index_candidate(query.predicate.as_ref())
    {
        return Ok(QueryPlan::IndexedLookup {
            table: query.table.clone(),
            columns: query.columns.clone(),
            column,
            value,
            predicate: residual,
        });
    }

    Ok(QueryPlan::SequentialScan {
        table: query.table.clone(),
        columns: query.columns.clone(),
        predicate: query.predicate.clone(),
    })
}

fn find_country_index_candidate(
    predicate: Option<&PredicateExpression>,
) -> Option<(String, String, Option<PredicateExpression>)> {
    let predicate = predicate?;

    match predicate {
        PredicateExpression::Comparison(predicate) => {
            if predicate.column.eq_ignore_ascii_case("country")
                && predicate.operator == ComparisonOperator::Equal
            {
                return Some((predicate.column.clone(), predicate.value.clone(), None));
            }

            None
        }

        PredicateExpression::And(left, right) => {
            if let Some(candidate) = find_country_index_candidate(Some(left)) {
                return Some((
                    candidate.0,
                    candidate.1,
                    combine_with_residual(candidate.2, Some((**right).clone())),
                ));
            }

            if let Some(candidate) = find_country_index_candidate(Some(right)) {
                return Some((
                    candidate.0,
                    candidate.1,
                    combine_with_residual(candidate.2, Some((**left).clone())),
                ));
            }

            None
        }

        PredicateExpression::Or(_, _) => None,
    }
}

fn combine_with_residual(
    existing: Option<PredicateExpression>,
    additional: Option<PredicateExpression>,
) -> Option<PredicateExpression> {
    match (existing, additional) {
        (None, None) => None,
        (Some(predicate), None) | (None, Some(predicate)) => Some(predicate),
        (Some(left), Some(right)) => {
            Some(PredicateExpression::And(Box::new(left), Box::new(right)))
        }
    }
}
