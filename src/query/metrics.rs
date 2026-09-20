use std::time::Duration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryExecutionMetrics {
    /// Wall-clock execution time measured by the executor.
    pub elapsed_ns: u128,

    /// Number of records entering predicate evaluation.
    pub rows_examined: usize,

    /// Number of records returned after predicate evaluation.
    pub rows_returned: usize,

    /// Physical page reads performed by the page store.
    pub page_reads: u64,

    /// Physical bytes read from the page store.
    pub bytes_read: u64,

    /// Number of index lookup operations performed.
    pub index_lookups: u64,

    /// Buffer-pool cache hits.
    pub cache_hits: u64,

    /// Buffer-pool cache misses.
    pub cache_misses: u64,
}

impl QueryExecutionMetrics {
    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos(self.elapsed_ns.min(u64::MAX as u128) as u64)
    }

    /// Calculate percentage difference between estimated and actual rows.
    ///
    /// Positive values mean the actual cardinality was higher than estimated.
    /// Negative values mean the actual cardinality was lower than estimated.
    pub fn cardinality_error_pct(&self, estimated_rows: usize) -> f64 {
        if estimated_rows == 0 {
            return if self.rows_returned == 0 {
                0.0
            } else {
                f64::INFINITY
            };
        }

        ((self.rows_returned as f64 - estimated_rows as f64) / estimated_rows as f64) * 100.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMetrics {
    pub estimated_rows: usize,
    pub estimated_cost: f64,
    pub actual: QueryExecutionMetrics,
}

impl QueryMetrics {
    pub fn new(estimated_rows: usize, estimated_cost: f64, actual: QueryExecutionMetrics) -> Self {
        Self {
            estimated_rows,
            estimated_cost,
            actual,
        }
    }

    pub fn cardinality_error_pct(&self) -> f64 {
        self.actual.cardinality_error_pct(self.estimated_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_metrics_are_zero() {
        let metrics = QueryExecutionMetrics::default();

        assert_eq!(metrics.elapsed_ns, 0);
        assert_eq!(metrics.rows_examined, 0);
        assert_eq!(metrics.rows_returned, 0);
        assert_eq!(metrics.page_reads, 0);
        assert_eq!(metrics.bytes_read, 0);
        assert_eq!(metrics.index_lookups, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
    }

    #[test]
    fn cardinality_error_reports_underestimate() {
        let metrics = QueryExecutionMetrics {
            rows_returned: 87,
            ..Default::default()
        };

        assert_eq!(metrics.cardinality_error_pct(10), 770.0);
    }

    #[test]
    fn cardinality_error_reports_overestimate() {
        let metrics = QueryExecutionMetrics {
            rows_returned: 10,
            ..Default::default()
        };

        assert_eq!(metrics.cardinality_error_pct(20), -50.0);
    }

    #[test]
    fn cardinality_error_is_zero_when_both_are_zero() {
        let metrics = QueryExecutionMetrics::default();

        assert_eq!(metrics.cardinality_error_pct(0), 0.0);
    }

    #[test]
    fn cardinality_error_is_infinite_for_zero_estimate() {
        let metrics = QueryExecutionMetrics {
            rows_returned: 10,
            ..Default::default()
        };

        assert!(metrics.cardinality_error_pct(0).is_infinite());
    }

    #[test]
    fn query_metrics_combine_estimate_and_actual() {
        let actual = QueryExecutionMetrics {
            rows_returned: 25,
            ..Default::default()
        };

        let metrics = QueryMetrics::new(10, 11.0, actual);

        assert_eq!(metrics.estimated_rows, 10);
        assert_eq!(metrics.estimated_cost, 11.0);
        assert_eq!(metrics.actual.rows_returned, 25);
        assert_eq!(metrics.cardinality_error_pct(), 150.0);
    }
}
