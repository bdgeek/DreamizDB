use crate::features::WorkloadFeature;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub target: String,
    pub confidence: f64,
    pub expected_benefit: f64,
    pub estimated_cost: f64,
    pub risk: String,
    pub model_version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub recommendation_target: String,
    pub latency_before_ms: f64,
    pub latency_after_ms: f64,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    pub confidence_boost: f64,
    pub successful_experiments: u64,
    pub failed_experiments: u64,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            confidence_boost: 0.0,
            successful_experiments: 0,
            failed_experiments: 0,
        }
    }
}

pub struct ModelStateStore {
    path: PathBuf,
}

impl ModelStateStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self) -> anyhow::Result<ModelState> {
        if !self.path.exists() {
            return Ok(ModelState::default());
        }

        let data = fs::read(&self.path)?;
        let state = serde_json::from_slice(&data)?;

        Ok(state)
    }

    pub fn save(&self, state: &ModelState) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = self.path.with_extension("tmp");

        let data = serde_json::to_vec_pretty(state)?;
        fs::write(&tmp, data)?;
        fs::rename(tmp, &self.path)?;

        Ok(())
    }
}

/// One completed adaptive-index experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub target: String,
    pub action: String,
    pub accepted: bool,
    pub latency_improvement_pct: f64,
    pub io_reduction_pct: f64,
    pub reward: f64,
    pub timestamp: String,
}

/// Persistent history of completed experiments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentHistory {
    pub experiments: Vec<ExperimentRecord>,
}

impl ExperimentHistory {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::default());
        }

        let data = fs::read(path)?;
        let history = serde_json::from_slice(&data)?;

        Ok(history)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = path.with_extension("tmp");

        let data = serde_json::to_vec_pretty(self)?;
        fs::write(&tmp, data)?;
        fs::rename(tmp, path)?;

        Ok(())
    }

    /// Identify previous experiments by recommendation target + action.
    pub fn find(&self, target: &str, action: &str) -> Vec<&ExperimentRecord> {
        self.experiments
            .iter()
            .filter(|experiment| experiment.target == target && experiment.action == action)
            .collect()
    }

    pub fn record(&mut self, experiment: ExperimentRecord) {
        self.experiments.push(experiment);
    }
}

/// Deterministic baseline predictor.
/// Learning is applied separately through ModelState.
pub fn predict_index(
    feature: &WorkloadFeature,
    predicate: &str,
    state: &ModelState,
) -> Recommendation {
    let frequency = *feature.predicate_frequency.get(predicate).unwrap_or(&0) as f64;

    let frequency_score = (frequency / 100.0).min(1.0);
    let latency_score = (feature.avg_latency_ms / 1000.0).min(1.0);

    let unindexed_score = if feature.executions == 0 {
        0.0
    } else {
        feature.unindexed_executions as f64 / feature.executions as f64
    };

    let selectivity_score = 1.0 - feature.avg_selectivity;

    let base_confidence = 0.40 * frequency_score
        + 0.25 * latency_score
        + 0.20 * unindexed_score
        + 0.15 * selectivity_score;

    let confidence = (base_confidence + state.confidence_boost).clamp(0.0, 0.99);

    let expected_benefit = (0.55 * latency_score + 0.45 * selectivity_score).clamp(0.0, 0.99);

    Recommendation {
        action: "CREATE_TEMP_INDEX".into(),
        target: predicate.into(),
        confidence,
        expected_benefit,
        estimated_cost: 1.0,
        risk: "LOW".into(),
        model_version: "baseline-v0.3-closed-loop".into(),
        reason: format!(
            "frequency={frequency:.0}, latency={:.1}ms, selectivity={:.3}, unindexed={unindexed_score:.2}, learned_boost={:.3}",
            feature.avg_latency_ms,
            feature.avg_selectivity,
            state.confidence_boost
        ),
    }
}

pub fn apply_feedback(state: &mut ModelState, feedback: &Feedback) {
    if feedback.accepted {
        state.successful_experiments += 1;

        let improvement = if feedback.latency_before_ms > 0.0 {
            ((feedback.latency_before_ms - feedback.latency_after_ms) / feedback.latency_before_ms)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };

        state.confidence_boost = (state.confidence_boost + 0.05 * improvement).clamp(0.0, 0.30);
    } else {
        state.failed_experiments += 1;

        state.confidence_boost = (state.confidence_boost - 0.02).clamp(0.0, 0.30);
    }
}

pub fn predict_heat(age_hours: f64, executions: f64, recent_ratio: f64) -> f64 {
    let recency = (-age_hours / 24.0).exp().clamp(0.0, 1.0);
    let frequency = (executions / 100.0).min(1.0);

    (0.45 * recency + 0.35 * frequency + 0.20 * recent_ratio).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_state_round_trip() {
        let path = std::env::temp_dir().join("dreamizdb-model-state-test.json");

        let store = ModelStateStore::new(&path);

        let state = ModelState {
            confidence_boost: 0.1234,
            successful_experiments: 7,
            failed_experiments: 2,
        };

        store.save(&state).expect("save model state");

        let loaded = store.load().expect("load model state");

        assert!((loaded.confidence_boost - state.confidence_boost).abs() < 1e-10);
        assert_eq!(loaded.successful_experiments, state.successful_experiments);
        assert_eq!(loaded.failed_experiments, state.failed_experiments);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn experiment_history_matches_target_and_action() {
        let mut history = ExperimentHistory::default();

        history.record(ExperimentRecord {
            target: "customers.country".into(),
            action: "CREATE_TEMP_INDEX".into(),
            accepted: true,
            latency_improvement_pct: 94.0,
            io_reduction_pct: 95.0,
            reward: 1.89,
            timestamp: "2026-08-31T00:00:00Z".into(),
        });

        history.record(ExperimentRecord {
            target: "orders.status".into(),
            action: "CREATE_TEMP_INDEX".into(),
            accepted: true,
            latency_improvement_pct: 80.0,
            io_reduction_pct: 90.0,
            reward: 1.70,
            timestamp: "2026-08-31T00:01:00Z".into(),
        });

        let matches = history.find("customers.country", "CREATE_TEMP_INDEX");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].accepted);
    }

    #[test]
    fn experiment_history_round_trip() {
        let path = std::env::temp_dir().join("dreamizdb-experiment-history-test.json");

        let mut history = ExperimentHistory::default();

        history.record(ExperimentRecord {
            target: "customers.country".into(),
            action: "CREATE_TEMP_INDEX".into(),
            accepted: true,
            latency_improvement_pct: 94.94,
            io_reduction_pct: 95.0,
            reward: 1.8994,
            timestamp: "2026-08-31T00:00:00Z".into(),
        });

        history.save(&path).expect("save history");

        let loaded = ExperimentHistory::load(&path).expect("load history");

        assert_eq!(loaded.experiments.len(), 1);
        assert_eq!(loaded.experiments[0].target, "customers.country");
        assert_eq!(loaded.experiments[0].action, "CREATE_TEMP_INDEX");
        assert!(loaded.experiments[0].accepted);

        let _ = std::fs::remove_file(path);
    }
}
