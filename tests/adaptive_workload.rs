use chrono::Utc;
use dreamizdb::{ai, optimizer, telemetry::QueryEvent};

fn event(predicate: &str, latency_ms: f64) -> QueryEvent {
    QueryEvent {
        timestamp: Utc::now(),
        sql: format!("SELECT * FROM customers WHERE {predicate}"),
        tables: vec!["customers".into()],
        columns: vec!["country".into()],
        predicates: vec![predicate.into()],
        rows_scanned: 10_000,
        rows_returned: 100,
        latency_ms,
        cpu_ms: latency_ms / 2.0,
        io_bytes: 10_000,
        index_used: None,
    }
}

#[test]
fn adaptive_workload_recommends_hot_predicate() {
    let mut events = Vec::new();

    // Hot workload.
    for _ in 0..150 {
        events.push(event("customers.country", 100.0));
    }

    let state = ai::ModelState::default();
    let features = dreamizdb::features::extract(&events);
    let feature = features
        .values()
        .next()
        .expect("workload feature should exist");

    let recommendation = ai::predict_index(feature, "customers.country", &state);

    assert_eq!(recommendation.target, "customers.country");

    assert_eq!(recommendation.action, "CREATE_TEMP_INDEX");
    assert_eq!(recommendation.target, "customers.country");
    assert!(recommendation.confidence >= 0.75);
    assert!(optimizer::validate_experiment(&recommendation));
}
