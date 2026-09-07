use anyhow::{anyhow, Result};

use crate::storage::persistence::PersistentTable;
use crate::storage::Record;

use super::planner::QueryPlan;
use super::types::{ComparisonOperator, PredicateExpression, PredicatePlan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRow {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
}

pub fn execute_query(table: &mut PersistentTable, plan: &QueryPlan) -> Result<QueryResult> {
    let (columns, predicate, records) = match plan {
        QueryPlan::SequentialScan {
            columns, predicate, ..
        } => (columns, predicate, table.scan_all()?),

        QueryPlan::IndexedLookup {
            columns,
            predicate,
            column,
            value,
            ..
        } => {
            if !column.eq_ignore_ascii_case("country") {
                return Err(anyhow!("unsupported index column: {column}"));
            }

            (columns, predicate, table.indexed_lookup(value)?)
        }
    };

    let rows = records
        .into_iter()
        .filter(|record| {
            predicate
                .as_ref()
                .map(|expression| evaluate_predicate(record, expression))
                .unwrap_or(true)
        })
        .map(|record| project_record(&record, columns))
        .collect::<Result<Vec<_>>>()?;

    Ok(QueryResult {
        columns: columns.clone(),
        rows,
    })
}

fn evaluate_predicate(record: &Record, predicate: &PredicateExpression) -> bool {
    match predicate {
        PredicateExpression::Comparison(comparison) => evaluate_comparison(record, comparison),

        PredicateExpression::And(left, right) => {
            evaluate_predicate(record, left) && evaluate_predicate(record, right)
        }

        PredicateExpression::Or(left, right) => {
            evaluate_predicate(record, left) || evaluate_predicate(record, right)
        }
    }
}

fn evaluate_comparison(record: &Record, comparison: &PredicatePlan) -> bool {
    match comparison.column.to_ascii_lowercase().as_str() {
        "country" => compare_strings(&record.country, comparison.operator, &comparison.value),

        "id" => {
            let value = match comparison.value.parse::<f64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            compare_numbers(record.id as f64, comparison.operator, value)
        }

        "value" => {
            let value = match comparison.value.parse::<f64>() {
                Ok(value) => value,
                Err(_) => return false,
            };

            compare_numbers(record.value, comparison.operator, value)
        }

        _ => false,
    }
}

fn compare_strings(left: &str, operator: ComparisonOperator, right: &str) -> bool {
    match operator {
        ComparisonOperator::Equal => left == right,
        ComparisonOperator::NotEqual => left != right,
        ComparisonOperator::GreaterThan => left > right,
        ComparisonOperator::GreaterThanOrEqual => left >= right,
        ComparisonOperator::LessThan => left < right,
        ComparisonOperator::LessThanOrEqual => left <= right,
    }
}

fn compare_numbers(left: f64, operator: ComparisonOperator, right: f64) -> bool {
    match operator {
        ComparisonOperator::Equal => left == right,
        ComparisonOperator::NotEqual => left != right,
        ComparisonOperator::GreaterThan => left > right,
        ComparisonOperator::GreaterThanOrEqual => left >= right,
        ComparisonOperator::LessThan => left < right,
        ComparisonOperator::LessThanOrEqual => left <= right,
    }
}

fn project_record(record: &Record, columns: &[String]) -> Result<QueryRow> {
    if columns.len() == 1 && columns[0] == "*" {
        return Ok(QueryRow {
            values: vec![format_record(record)],
        });
    }

    let mut values = Vec::with_capacity(columns.len());

    for column in columns {
        match column.to_ascii_lowercase().as_str() {
            "id" => values.push(record.id.to_string()),
            "country" => values.push(record.country.clone()),
            "value" => values.push(format_number(record.value)),
            _ => return Err(anyhow!("unknown column: {column}")),
        }
    }

    Ok(QueryRow { values })
}

fn format_record(record: &Record) -> String {
    format!(
        "{},{},{}",
        record.id,
        record.country,
        format_number(record.value)
    )
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}
