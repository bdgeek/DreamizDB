use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvent {
    pub timestamp: DateTime<Utc>,
    pub sql: String,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub predicates: Vec<String>,
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub latency_ms: f64,
    pub cpu_ms: f64,
    pub io_bytes: u64,
    pub index_used: Option<String>,
}

impl QueryEvent {
    /// Stable baseline fingerprint. Full SQL AST normalization is a later milestone.
    pub fn fingerprint(&self) -> String {
        let normalized = self
            .sql
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let mut h = Sha256::new();
        h.update(normalized.as_bytes());
        format!("{:x}", h.finalize())
    }

    pub fn selectivity(&self) -> f64 {
        if self.rows_scanned == 0 {
            1.0
        } else {
            (self.rows_returned as f64 / self.rows_scanned as f64).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryStore {
    path: String,
}

impl TelemetryStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().into_owned(),
        }
    }

    pub fn append(&self, event: &QueryEvent) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", serde_json::to_string(event)?)?;
        Ok(())
    }

    pub fn load(&self) -> anyhow::Result<Vec<QueryEvent>> {
        if !Path::new(&self.path).exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.path)?;
        BufReader::new(file)
            .lines()
            .map(|line| Ok(serde_json::from_str(&line?)?))
            .collect()
    }
}
