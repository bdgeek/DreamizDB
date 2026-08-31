use crate::ai::Recommendation;

/// Determines whether a recommendation is safe to test experimentally.
/// This does NOT authorize permanent adoption.
pub fn validate_experiment(r: &Recommendation) -> bool {
    r.action == "CREATE_TEMP_INDEX"
        && r.confidence >= 0.50
        && r.expected_benefit > 0.10
        && r.estimated_cost <= 10.0
        && r.risk == "LOW"
}

/// Final physical-change authorization must come from measured experiment results.
pub fn validate_result(latency_improvement_pct: f64, io_reduction_pct: f64) -> bool {
    latency_improvement_pct > 5.0 && io_reduction_pct >= 0.0
}
