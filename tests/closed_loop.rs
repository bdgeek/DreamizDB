use dreamizdb::ai::{apply_feedback, predict_index, Feedback, ModelState};
use dreamizdb::features::WorkloadFeature;

#[test]
fn closed_loop_learns_from_successful_experiment() {
    let mut feature = WorkloadFeature::default();

    feature.executions = 100;
    feature.avg_latency_ms = 900.0;
    feature.avg_selectivity = 0.05;
    feature.unindexed_executions = 100;
    feature
        .predicate_frequency
        .insert("customers.country".into(), 100);

    let mut state = ModelState::default();

    let before = predict_index(&feature, "customers.country", &state);

    assert!(before.expected_benefit > 0.10);

    let feedback = Feedback {
        recommendation_target: before.target.clone(),
        latency_before_ms: 900.0,
        latency_after_ms: 100.0,
        accepted: true,
    };

    apply_feedback(&mut state, &feedback);

    assert_eq!(state.successful_experiments, 1);
    assert!(state.confidence_boost > 0.0);

    let after = predict_index(&feature, "customers.country", &state);

    assert!(after.confidence > before.confidence);
}
