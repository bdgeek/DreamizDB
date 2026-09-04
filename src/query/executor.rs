use super::planner::QueryPlan;
use super::types::{ComparisonOperator, PredicateExpression};
use crate::storage::persistence::PersistentTable;
use crate::storage::Record;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRow {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
}

pub fn execute_query(table: &mut PersistentTable, plan: &QueryPlan) -> Result<QueryResult> {
    match plan {
        QueryPlan::SequentialScan {
            columns, predicate, ..
        } => {
            let records = table.scan_all()?;

            let records = match predicate {
                Some(predicate) => filter_records(records, predicate)?,
                None => records,
            };

            build_result(columns, records)
        }

        QueryPlan::IndexedLookup {
            columns,
            column,
            value,
            ..
        } => {
            if !column.eq_ignore_ascii_case("country") {
                return Err(anyhow!("indexed lookup is only supported for country"));
            }

            let records = table.indexed_lookup(value)?;

            build_result(columns, records)
        }
    }
}

fn filter_records(records: Vec<Record>, predicate: &PredicateExpression) -> Result<Vec<Record>> {
    records
        .into_iter()
        .map(|record| {
            if predicate_matches(&record, predicate)? {
                Ok(Some(record))
            } else {
                Ok(None)
            }
        })
        .filter_map(Result::transpose)
        .collect()
}

fn predicate_matches(record: &Record, predicate: &PredicateExpression) -> Result<bool> {
    match predicate {
        PredicateExpression::Comparison(predicate) => comparison_matches(record, predicate),

        PredicateExpression::And(left, right) => {
            let left_matches = predicate_matches(record, left)?;

            if !left_matches {
                return Ok(false);
            }

            predicate_matches(record, right)
        }

        PredicateExpression::Or(left, right) => {
            let left_matches = predicate_matches(record, left)?;

            if left_matches {
                return Ok(true);
            }

            predicate_matches(record, right)
        }
    }
}

fn comparison_matches(record: &Record, predicate: &super::types::PredicatePlan) -> Result<bool> {
    let record_value = match predicate.column.to_ascii_lowercase().as_str() {
        "id" => record.id.to_string(),
        "country" => record.country.clone(),
        "value" => format_number(record.value),
        _ => {
            return Err(anyhow!(
                "unsupported predicate column: {}",
                predicate.column
            ));
        }
    };

    match predicate.operator {
        ComparisonOperator::Equal => Ok(record_value == predicate.value),

        ComparisonOperator::NotEqual => Ok(record_value != predicate.value),

        ComparisonOperator::LessThan => {
            compare_values(&record_value, &predicate.value, |ordering| ordering.is_lt())
        }

        ComparisonOperator::LessThanOrEqual => {
            compare_values(&record_value, &predicate.value, |ordering| ordering.is_le())
        }

        ComparisonOperator::GreaterThan => {
            compare_values(&record_value, &predicate.value, |ordering| ordering.is_gt())
        }

        ComparisonOperator::GreaterThanOrEqual => {
            compare_values(&record_value, &predicate.value, |ordering| ordering.is_ge())
        }
    }
}

fn compare_values<F>(left: &str, right: &str, comparison: F) -> Result<bool>
where
    F: FnOnce(std::cmp::Ordering) -> bool,
{
    if let (Ok(left_number), Ok(right_number)) = (left.parse::<f64>(), right.parse::<f64>()) {
        return Ok(comparison(left_number.total_cmp(&right_number)));
    }

    Ok(comparison(left.cmp(right)))
}

fn build_result(columns: &[String], records: Vec<Record>) -> Result<QueryResult> {
    let rows = records
        .into_iter()
        .map(|record| {
            let values = columns
                .iter()
                .map(|column| project_column(&record, column))
                .collect::<Result<Vec<_>>>()?;

            Ok(QueryRow { values })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(QueryResult {
        columns: columns.to_vec(),
        rows,
    })
}

fn project_column(record: &Record, column: &str) -> Result<String> {
    match column.to_ascii_lowercase().as_str() {
        "*" => Ok(format!(
            "{},{},{}",
            record.id,
            record.country,
            format_number(record.value)
        )),

        "id" => Ok(record.id.to_string()),

        "country" => Ok(record.country.clone()),

        "value" => Ok(format_number(record.value)),

        _ => Err(anyhow!("unsupported selected column: {column}")),
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
