use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMeasurement {
    pub latency_ms: f64,
    pub io_units: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub target: String,
    pub before: BenchmarkMeasurement,
    pub after: BenchmarkMeasurement,
    pub latency_improvement_pct: f64,
    pub io_reduction_pct: f64,
    pub accepted: bool,
    pub reward: f64,
}

pub fn evaluate(
    target: impl Into<String>,
    before: BenchmarkMeasurement,
    after: BenchmarkMeasurement,
) -> ExperimentResult {
    let latency_improvement_pct = if before.latency_ms > 0.0 {
        ((before.latency_ms - after.latency_ms) / before.latency_ms) * 100.0
    } else {
        0.0
    };

    let io_reduction_pct = if before.io_units > 0.0 {
        ((before.io_units - after.io_units) / before.io_units) * 100.0
    } else {
        0.0
    };

    let accepted = crate::optimizer::validate_result(latency_improvement_pct, io_reduction_pct);
    let reward = (latency_improvement_pct.max(0.0) / 100.0) + (io_reduction_pct.max(0.0) / 100.0);

    ExperimentResult {
        target: target.into(),
        before,
        after,
        latency_improvement_pct,
        io_reduction_pct,
        accepted,
        reward,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_meaningful_improvement() {
        let result = evaluate(
            "customers.country",
            BenchmarkMeasurement {
                latency_ms: 100.0,
                io_units: 1000.0,
            },
            BenchmarkMeasurement {
                latency_ms: 60.0,
                io_units: 700.0,
            },
        );
        assert!(result.accepted);
        assert!(result.reward > 0.0);
    }

    #[test]
    fn rejects_insufficient_improvement() {
        let result = evaluate(
            "customers.country",
            BenchmarkMeasurement {
                latency_ms: 100.0,
                io_units: 1000.0,
            },
            BenchmarkMeasurement {
                latency_ms: 98.0,
                io_units: 990.0,
            },
        );
        assert!(!result.accepted);
    }
}
