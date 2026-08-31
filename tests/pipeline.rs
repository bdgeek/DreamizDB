use chrono::Utc;
use dreamizdb::{
    ai, features, optimizer, storage,
    telemetry::{QueryEvent, TelemetryStore},
};
use std::fs;

fn event(sql: &str) -> QueryEvent {
    QueryEvent {
        timestamp: Utc::now(),
        sql: sql.into(),
        tables: vec![],
        columns: vec![],
        predicates: vec!["t.x".into()],
        rows_scanned: 100,
        rows_returned: 10,
        latency_ms: 100.0,
        cpu_ms: 20.0,
        io_bytes: 1000,
        index_used: None,
    }
}

#[test]
fn fingerprint_is_stable_for_case_and_whitespace() {
    assert_eq!(
        event("SELECT * FROM t WHERE x = 1").fingerprint(),
        event("  select  *  from t where x = 1  ").fingerprint()
    );
}

#[test]
fn feature_extraction_counts_workload() {
    let f = features::extract(&[
        event("SELECT * FROM t WHERE x=1"),
        event("SELECT * FROM t WHERE x=1"),
    ]);
    assert_eq!(f.len(), 1);
    assert_eq!(f.values().next().unwrap().executions, 2);
}

#[test]
fn predictor_and_gate_work() {
    let events = (0..150)
        .map(|_| event("SELECT * FROM t WHERE x=1"))
        .collect::<Vec<_>>();
    let f = features::extract(&events);
    let state = ai::ModelState::default();

    let r = ai::predict_index(f.values().next().unwrap(), "t.x", &state);
    assert!(r.confidence >= 0.75);
    assert!(optimizer::validate_experiment(&r));
}

#[test]
fn telemetry_round_trip() {
    let path = std::env::temp_dir().join(format!("dreamizdb-{}.jsonl", std::process::id()));
    let store = TelemetryStore::new(&path);
    store.append(&event("SELECT 1")).unwrap();
    assert_eq!(store.load().unwrap().len(), 1);
    fs::remove_file(path).ok();
}

#[test]
fn heat_maps_to_tiers() {
    assert_eq!(storage::tier_for_heat(0.95), storage::Tier::Hot);
    assert_eq!(storage::tier_for_heat(0.02), storage::Tier::Archive);
}
