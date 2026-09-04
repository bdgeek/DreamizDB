use super::types::{ComparisonOperator, SelectQuery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPlan {
    SequentialScan {
        table: String,
        columns: Vec<String>,
        predicate: Option<PredicatePlan>,
    },
    IndexedLookup {
        table: String,
        columns: Vec<String>,
        column: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicatePlan {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: String,
}

pub fn plan_query(query: &SelectQuery, has_country_index: bool) -> anyhow::Result<QueryPlan> {
    if query.table.is_empty() {
        anyhow::bail!("query table cannot be empty");
    }

    if let Some(predicate) = &query.predicate {
        if predicate.column.eq_ignore_ascii_case("country")
            && predicate.operator == ComparisonOperator::Equal
            && has_country_index
        {
            return Ok(QueryPlan::IndexedLookup {
                table: query.table.clone(),
                columns: query.columns.clone(),
                column: predicate.column.clone(),
                value: predicate.value.clone(),
            });
        }
    }

    let predicate = query.predicate.as_ref().map(|predicate| PredicatePlan {
        column: predicate.column.clone(),
        operator: predicate.operator.clone(),
        value: predicate.value.clone(),
    });

    Ok(QueryPlan::SequentialScan {
        table: query.table.clone(),
        columns: query.columns.clone(),
        predicate,
    })
}
