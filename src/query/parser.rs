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

/// Parses boolean predicate expressions with:
///
/// - comparisons
/// - AND
/// - OR
/// - parentheses
///
/// AND has higher precedence than OR.
///
/// Examples:
///
/// country = 'BD'
/// country = 'BD' AND value > 10
/// country = 'BD' OR country = 'IN'
/// country = 'BD' AND (value > 10 OR value < 5)
/// (country = 'BD' OR country = 'IN') AND value >= 10
fn parse_predicate_expression(condition: &str) -> anyhow::Result<PredicateExpression> {
    let condition = condition.trim();

    if condition.is_empty() {
        anyhow::bail!("missing predicate");
    }

    let condition = strip_outer_parentheses(condition)?;

    // OR has the lowest precedence, so split it first.
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

    // AND has higher precedence than OR.
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

    // At this point the expression must be a simple comparison.
    if contains_parentheses(condition) {
        anyhow::bail!("unbalanced or invalid parentheses");
    }

    Ok(PredicateExpression::Comparison(parse_predicate(condition)?))
}

/// Removes one or more complete outer parenthesis pairs.
///
/// Example:
///
/// ((country = 'BD'))
///
/// becomes:
///
/// country = 'BD'
fn strip_outer_parentheses(condition: &str) -> anyhow::Result<&str> {
    let mut expression = condition.trim();

    loop {
        if !is_wrapped_by_outer_parentheses(expression)? {
            return Ok(expression);
        }

        expression = expression[1..expression.len() - 1].trim();
    }
}

/// Returns true when the entire expression is enclosed by one matching
/// outer pair of parentheses.
fn is_wrapped_by_outer_parentheses(expression: &str) -> anyhow::Result<bool> {
    let expression = expression.trim();

    if expression.len() < 2 || !expression.starts_with('(') || !expression.ends_with(')') {
        return Ok(false);
    }

    let bytes = expression.as_bytes();
    let mut depth = 0usize;
    let mut in_quotes = false;

    for (index, &byte) in bytes.iter().enumerate() {
        match byte {
            b'\'' => {
                in_quotes = !in_quotes;
            }

            b'(' if !in_quotes => {
                depth += 1;
            }

            b')' if !in_quotes => {
                if depth == 0 {
                    anyhow::bail!("unbalanced parentheses");
                }

                depth -= 1;

                // If the outer pair closes before the final character,
                // then the whole expression is not wrapped by it.
                if depth == 0 && index != bytes.len() - 1 {
                    return Ok(false);
                }
            }

            _ => {}
        }
    }

    if in_quotes || depth != 0 {
        anyhow::bail!("unbalanced parentheses");
    }

    Ok(true)
}

/// Finds AND/OR only when the operator is at the top expression level.
///
/// Logical operators inside parentheses or quoted strings are ignored.
fn find_logical_operator(condition: &str, operator: &str) -> Option<usize> {
    let bytes = condition.as_bytes();
    let operator_bytes = operator.as_bytes();

    if operator_bytes.is_empty() || bytes.len() < operator_bytes.len() {
        return None;
    }

    let mut depth = 0usize;
    let mut in_quotes = false;
    let mut index = 0usize;

    while index + operator_bytes.len() <= bytes.len() {
        match bytes[index] {
            b'\'' => {
                in_quotes = !in_quotes;
                index += 1;
                continue;
            }

            b'(' if !in_quotes => {
                depth += 1;
                index += 1;
                continue;
            }

            b')' if !in_quotes => {
                if depth == 0 {
                    return None;
                }

                depth -= 1;
                index += 1;
                continue;
            }

            _ => {}
        }

        if !in_quotes && depth == 0 && &bytes[index..index + operator_bytes.len()] == operator_bytes
        {
            return Some(index);
        }

        index += 1;
    }

    None
}

fn contains_parentheses(condition: &str) -> bool {
    condition.contains('(') || condition.contains(')')
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

    if column.contains('(') || column.contains(')') {
        anyhow::bail!("invalid predicate column");
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
