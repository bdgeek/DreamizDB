use super::planner::{PredicatePlan, QueryPlan};
use super::types::ComparisonOperator;
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
            let records = match predicate {
                Some(predicate) => {
                    let country = country_predicate_value(predicate)?;
                    table.sequential_scan(&country)?
                }
                None => table.scan_all()?,
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

fn country_predicate_value(predicate: &PredicatePlan) -> Result<String> {
    match predicate.operator {
        ComparisonOperator::Equal => {
            if !predicate.column.eq_ignore_ascii_case("country") {
                return Err(anyhow!(
                    "sequential predicate is only supported for country"
                ));
            }

            Ok(predicate.value.clone())
        }
    }
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
