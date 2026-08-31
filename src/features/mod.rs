use crate::telemetry::QueryEvent;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct WorkloadFeature {
    pub executions: u64,
    pub avg_latency_ms: f64,
    pub avg_selectivity: f64,
    pub total_io_bytes: u64,
    pub total_rows_scanned: u64,
    pub total_rows_returned: u64,
    pub predicate_frequency: HashMap<String, u64>,
    pub unindexed_executions: u64,
}

pub fn extract(events: &[QueryEvent]) -> HashMap<String, WorkloadFeature> {
    let mut out = HashMap::new();
    for e in events {
        let f = out
            .entry(e.fingerprint())
            .or_insert_with(WorkloadFeature::default);
        f.executions += 1;
        let n = f.executions as f64;
        f.avg_latency_ms += (e.latency_ms - f.avg_latency_ms) / n;
        f.avg_selectivity += (e.selectivity() - f.avg_selectivity) / n;
        f.total_io_bytes += e.io_bytes;
        f.total_rows_scanned += e.rows_scanned;
        f.total_rows_returned += e.rows_returned;
        if e.index_used.is_none() {
            f.unindexed_executions += 1;
        }
        for p in &e.predicates {
            *f.predicate_frequency.entry(p.clone()).or_insert(0) += 1;
        }
    }
    out
}

pub fn window(events: &[QueryEvent], end: DateTime<Utc>, minutes: i64) -> Vec<QueryEvent> {
    let start = end - Duration::minutes(minutes);
    events
        .iter()
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .cloned()
        .collect()
}
