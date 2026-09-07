#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueryStatistics {
    pub total_rows: usize,
    pub indexed_rows: usize,
    pub matching_rows: usize,
}

impl QueryStatistics {
    pub fn new(total_rows: usize, indexed_rows: usize, matching_rows: usize) -> Self {
        Self {
            total_rows,
            indexed_rows,
            matching_rows,
        }
    }

    /// Estimated cost of reading the complete table sequentially.
    pub fn sequential_cost(&self) -> f64 {
        self.total_rows as f64
    }

    /// Estimated cost of using the index.
    ///
    /// The index path has:
    /// - a fixed lookup/traversal cost;
    /// - index entries that must be examined;
    /// - matching rows that must be fetched.
    ///
    /// The indexed-row component prevents a high-cardinality index
    /// from being preferred merely because it exists.
    pub fn indexed_cost(&self) -> f64 {
        if self.indexed_rows == 0 {
            return f64::INFINITY;
        }

        let lookup_cost = 1.0;
        let index_traversal_cost = (self.indexed_rows as f64).sqrt();
        let row_fetch_cost = self.matching_rows as f64;

        lookup_cost + index_traversal_cost + row_fetch_cost
    }

    /// Choose the index only when its estimated cost is strictly
    /// lower than a sequential scan.
    pub fn should_use_index(&self) -> bool {
        self.indexed_rows > 0 && self.indexed_cost() < self.sequential_cost()
    }
}
