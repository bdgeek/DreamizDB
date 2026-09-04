use super::types::{ComparisonOperator, Predicate, SelectQuery};

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

        (table, Some(parse_predicate(condition)?))
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

fn parse_predicate(condition: &str) -> anyhow::Result<Predicate> {
    let operators = [
        ("<=", ComparisonOperator::LessThanOrEqual),
        (">=", ComparisonOperator::GreaterThanOrEqual),
        ("!=", ComparisonOperator::NotEqual),
        ("=", ComparisonOperator::Equal),
        ("<", ComparisonOperator::LessThan),
        (">", ComparisonOperator::GreaterThan),
    ];

    let (operator_text, operator, position) = operators
        .iter()
        .filter_map(|(text, operator)| {
            condition
                .find(text)
                .map(|position| (*text, operator.clone(), position))
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

    Ok(Predicate {
        column: column.to_owned(),
        operator,
        value,
    })
}
