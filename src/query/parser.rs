use super::types::{ComparisonOperator, PredicateExpression, PredicatePlan, SelectQuery};

pub fn parse_select(sql: &str) -> anyhow::Result<SelectQuery> {
    let sql = sql.trim().trim_end_matches(';').trim();

    let upper = sql.to_uppercase();

    if !upper.starts_with("SELECT ") {
        anyhow::bail!("only SELECT queries are supported");
    }

    let from_pos = upper
        .find(" FROM ")
        .ok_or_else(|| anyhow::anyhow!("missing FROM clause"))?;

    let columns_part = sql["SELECT ".len()..from_pos].trim();

    if columns_part.is_empty() {
        anyhow::bail!("missing selected columns");
    }

    let after_from = &sql[from_pos + " FROM ".len()..];
    let after_from_upper = after_from.to_uppercase();

    let (table, predicate) = if let Some(where_pos) = after_from_upper.find(" WHERE ") {
        let table = after_from[..where_pos].trim();

        if table.is_empty() {
            anyhow::bail!("missing table name");
        }

        let condition = after_from[where_pos + " WHERE ".len()..].trim();

        if condition.is_empty() {
            anyhow::bail!("missing predicate");
        }

        (table, Some(parse_predicate_expression(condition)?))
    } else {
        let table = after_from.trim();

        if table.is_empty() {
            anyhow::bail!("missing table name");
        }

        (table, None)
    };

    let columns = columns_part
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(SelectQuery {
        columns,
        table: table.to_owned(),
        predicate,
    })
}

fn parse_predicate_expression(condition: &str) -> anyhow::Result<PredicateExpression> {
    let condition = condition.trim();

    if let Some(position) = find_logical_operator(condition, " OR ") {
        let left = condition[..position].trim();
        let right = condition[position + " OR ".len()..].trim();

        if left.is_empty() || right.is_empty() {
            anyhow::bail!("invalid OR expression");
        }

        return Ok(PredicateExpression::Or(
            Box::new(parse_predicate_expression(left)?),
            Box::new(parse_predicate_expression(right)?),
        ));
    }

    if let Some(position) = find_logical_operator(condition, " AND ") {
        let left = condition[..position].trim();
        let right = condition[position + " AND ".len()..].trim();

        if left.is_empty() || right.is_empty() {
            anyhow::bail!("invalid AND expression");
        }

        return Ok(PredicateExpression::And(
            Box::new(parse_predicate_expression(left)?),
            Box::new(parse_predicate_expression(right)?),
        ));
    }

    Ok(PredicateExpression::Comparison(parse_predicate(condition)?))
}

fn find_logical_operator(condition: &str, operator: &str) -> Option<usize> {
    condition.to_uppercase().find(operator)
}

fn parse_predicate(condition: &str) -> anyhow::Result<PredicatePlan> {
    let operators = [
        ("<=", ComparisonOperator::LessThanOrEqual),
        (">=", ComparisonOperator::GreaterThanOrEqual),
        ("!=", ComparisonOperator::NotEqual),
        ("<>", ComparisonOperator::NotEqual),
        ("=", ComparisonOperator::Equal),
        ("<", ComparisonOperator::LessThan),
        (">", ComparisonOperator::GreaterThan),
    ];

    let (operator_text, operator, position) = operators
        .iter()
        .filter_map(|(text, operator)| {
            condition
                .find(text)
                .map(|position| (*text, *operator, position))
        })
        .min_by_key(|(_, _, position)| *position)
        .ok_or_else(|| anyhow::anyhow!("unsupported comparison operator"))?;

    let column = condition[..position].trim();

    if column.is_empty() {
        anyhow::bail!("missing predicate column");
    }

    let value = condition[position + operator_text.len()..].trim();

    if value.is_empty() {
        anyhow::bail!("missing predicate value");
    }

    let value = value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(value)
        .to_owned();

    Ok(PredicatePlan {
        column: column.to_owned(),
        operator,
        value,
    })
}
