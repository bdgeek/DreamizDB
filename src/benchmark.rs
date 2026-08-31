use crate::storage::persistence::PersistentTable;
use crate::storage::{Record, Table};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub elapsed_ns: u128,
    pub rows: usize,
    pub queries: usize,
    pub pages_read: u64,
    pub bytes_read: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
}

impl BenchmarkResult {
    pub fn avg_page_reads(&self) -> f64 {
        self.pages_read as f64 / self.queries.max(1) as f64
    }
    pub fn avg_bytes_read(&self) -> f64 {
        self.bytes_read as f64 / self.queries.max(1) as f64
    }
}

pub fn benchmark_scan(table: &Table, country: &str, iterations: usize) -> BenchmarkResult {
    let start = Instant::now();
    let mut rows = 0;
    for _ in 0..iterations {
        rows = table.sequential_scan(country).len();
    }
    BenchmarkResult {
        operation: "sequential_scan".into(),
        elapsed_ns: start.elapsed().as_nanos(),
        rows,
        queries: iterations,
        pages_read: 0,
        bytes_read: 0,
        cache_hits: 0,
        cache_misses: 0,
        evictions: 0,
    }
}

pub fn benchmark_index(table: &Table, country: &str, iterations: usize) -> BenchmarkResult {
    let start = Instant::now();
    let mut rows = 0;
    for _ in 0..iterations {
        rows = table.indexed_lookup(country).map(|v| v.len()).unwrap_or(0);
    }
    BenchmarkResult {
        operation: "indexed_lookup".into(),
        elapsed_ns: start.elapsed().as_nanos(),
        rows,
        queries: iterations,
        pages_read: 0,
        bytes_read: 0,
        cache_hits: 0,
        cache_misses: 0,
        evictions: 0,
    }
}

pub fn benchmark_persistent_scan(
    table: &mut PersistentTable,
    country: &str,
    iterations: usize,
) -> anyhow::Result<BenchmarkResult> {
    table.clear_cache();
    table.reset_metrics();
    let start = Instant::now();
    let mut rows = 0;
    for _ in 0..iterations {
        rows = table.sequential_scan(country)?.len();
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let io = table.io_stats();
    let cache = table.cache_stats();
    Ok(BenchmarkResult {
        operation: "persistent_sequential_scan".into(),
        elapsed_ns,
        rows,
        queries: iterations,
        pages_read: io.reads,
        bytes_read: io.bytes_read,
        cache_hits: cache.hits,
        cache_misses: cache.misses,
        evictions: cache.evictions,
    })
}

pub fn benchmark_persistent_index(
    table: &mut PersistentTable,
    country: &str,
    iterations: usize,
) -> anyhow::Result<BenchmarkResult> {
    table.clear_cache();
    table.reset_metrics();
    let start = Instant::now();
    let mut rows = 0;
    for _ in 0..iterations {
        rows = table.indexed_lookup(country)?.len();
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let io = table.io_stats();
    let cache = table.cache_stats();
    Ok(BenchmarkResult {
        operation: "persistent_indexed_lookup".into(),
        elapsed_ns,
        rows,
        queries: iterations,
        pages_read: io.reads,
        bytes_read: io.bytes_read,
        cache_hits: cache.hits,
        cache_misses: cache.misses,
        evictions: cache.evictions,
    })
}

pub fn sample_table(size: u64) -> Table {
    let mut table = Table::new();
    for id in 0..size {
        let country = if id < size / 20 {
            "BD"
        } else if id < size / 5 {
            "IN"
        } else {
            "US"
        };
        table.insert(Record {
            id,
            country: country.into(),
            value: id as f64,
        });
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_paths_return_same_cardinality() {
        let mut table = sample_table(10_000);
        let scan = benchmark_scan(&table, "BD", 3);
        table.build_country_index();
        let indexed = benchmark_index(&table, "BD", 3);
        assert_eq!(scan.rows, indexed.rows);
        assert!(scan.elapsed_ns > 0);
    }
}

pub fn benchmark_restart_index(
    path: &std::path::Path,
    country: &str,
) -> anyhow::Result<(usize, u64, u64)> {
    let mut table = PersistentTable::open(path)?;
    table.clear_cache();
    table.reset_metrics();
    let rows = table.indexed_lookup(country)?.len();
    let io = table.io_stats();
    let cache = table.cache_stats();
    Ok((rows, io.reads, cache.misses))
}
