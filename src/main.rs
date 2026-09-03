mod ai;
mod benchmark;
mod experiment;
mod features;
mod optimizer;
mod storage;
mod telemetry;

use crate::ai::{
    apply_feedback, predict_heat, predict_index, ExperimentHistory, ExperimentRecord, Feedback,
    ModelStateStore,
};
use crate::benchmark::{
    benchmark_persistent_index, benchmark_persistent_scan, benchmark_restart_index, sample_table,
};
use crate::experiment::{evaluate, BenchmarkMeasurement};
use crate::storage::persistence::PersistentTable;
use crate::storage::tier_for_heat;
use crate::telemetry::{QueryEvent, TelemetryStore};
use chrono::Utc;

fn main() {
    println!("DreamizDB v0.9 — Persistent Adaptive Index Engine");

    // All runtime state is kept inside the project-local data directory.
    let data_dir = std::env::current_dir()
        .expect("get current directory")
        .join("data");

    std::fs::create_dir_all(&data_dir).expect("create data directory");

    let telemetry_path = data_dir.join("telemetry.jsonl");
    let model_path = data_dir.join("model-state.json");
    let history_path = data_dir.join("experiment-history.json");
    let database_path = data_dir.join("customers.db");

    let telemetry_store = TelemetryStore::new(&telemetry_path);
    let model_store = ModelStateStore::new(&model_path);

    let mut historical_events = telemetry_store.load().expect("load telemetry");

    let mut experiment_history =
        ExperimentHistory::load(&history_path).expect("load experiment history");

    println!();
    println!("=== Storage ===");
    println!("data directory: {}", data_dir.display());
    println!("database: {}", database_path.display());
    println!("telemetry: {}", telemetry_path.display());
    println!("model state: {}", model_path.display());
    println!("experiment history: {}", history_path.display());
    println!(
        "recorded experiments: {}",
        experiment_history.experiments.len()
    );

    // Synthetic workload used by the current engine validation.
    let events = vec![
        QueryEvent {
            timestamp: Utc::now(),
            sql: "SELECT * FROM customers WHERE country = 'BD'".into(),
            tables: vec!["customers".into()],
            columns: vec!["country".into()],
            predicates: vec!["customers.country".into()],
            rows_scanned: 100_000,
            rows_returned: 5_000,
            latency_ms: 749.037,
            cpu_ms: 400.0,
            io_bytes: 100_000 * 4096,
            index_used: None,
        },
        QueryEvent {
            timestamp: Utc::now(),
            sql: "SELECT * FROM customers WHERE country = 'BD'".into(),
            tables: vec!["customers".into()],
            columns: vec!["country".into()],
            predicates: vec!["customers.country".into()],
            rows_scanned: 100_000,
            rows_returned: 5_000,
            latency_ms: 749.037,
            cpu_ms: 400.0,
            io_bytes: 100_000 * 4096,
            index_used: None,
        },
        QueryEvent {
            timestamp: Utc::now(),
            sql: "SELECT * FROM customers WHERE country = 'BD'".into(),
            tables: vec!["customers".into()],
            columns: vec!["country".into()],
            predicates: vec!["customers.country".into()],
            rows_scanned: 100_000,
            rows_returned: 5_000,
            latency_ms: 749.037,
            cpu_ms: 400.0,
            io_bytes: 100_000 * 4096,
            index_used: None,
        },
    ];

    // Persist telemetry so future runs accumulate workload data.
    for event in &events {
        telemetry_store.append(event).expect("append telemetry");
        historical_events.push(event.clone());
    }

    let workload_window = crate::features::window(&historical_events, Utc::now(), 60);
    let features = crate::features::extract(&workload_window);
    let feature = features.values().next().expect("workload feature");

    let newest_event = workload_window
        .iter()
        .map(|event| event.timestamp)
        .max()
        .expect("workload window is not empty");

    let age_hours = (Utc::now() - newest_event).num_seconds().max(0) as f64 / 3600.0;

    let recent_ratio = if historical_events.is_empty() {
        1.0
    } else {
        (workload_window.len() as f64 / historical_events.len() as f64).clamp(0.0, 1.0)
    };

    let heat = predict_heat(age_hours, feature.executions as f64, recent_ratio);
    let tier = tier_for_heat(heat);

    // Load learned model state.
    let mut model_state = model_store.load().expect("load model state");

    println!();
    println!("=== Model State ===");
    println!(
        "successful experiments: {}",
        model_state.successful_experiments
    );
    println!("failed experiments: {}", model_state.failed_experiments);
    println!("confidence boost: {:.4}", model_state.confidence_boost);

    println!();
    println!("=== Workload Heat ===");
    println!("age: {:.4} hours", age_hours);
    println!("executions: {}", feature.executions);
    println!("recent ratio: {:.4}", recent_ratio);
    println!("heat: {:.4}", heat);
    println!("storage tier: {:?}", tier);

    // AI proposes an optimization.
    let recommendation = predict_index(feature, "customers.country", &model_state);

    println!();
    println!("=== AI Recommendation ===");
    println!("action: {}", recommendation.action);
    println!("target: {}", recommendation.target);
    println!("confidence: {:.4}", recommendation.confidence);
    println!("expected benefit: {:.4}", recommendation.expected_benefit);
    println!("estimated cost: {:.4}", recommendation.estimated_cost);
    println!("risk: {}", recommendation.risk);
    println!("model: {}", recommendation.model_version);
    println!("reason: {}", recommendation.reason);

    // Check previous experiments for this target + action.
    let previous_experiments =
        experiment_history.find(&recommendation.target, &recommendation.action);

    println!(
        "previous experiments for target/action: {}",
        previous_experiments.len()
    );

    // Safety gate: recommendation may enter an experiment.
    let experiment_allowed = crate::optimizer::validate_experiment(&recommendation);

    println!(
        "experiment gate: {}",
        if experiment_allowed { "PASS" } else { "REJECT" }
    );

    if !experiment_allowed {
        println!("decision: NO EXPERIMENT");
        return;
    }

    // Create persistent database only on the first run.
    if !database_path.exists() {
        let table = sample_table(100_000);

        PersistentTable::create_from_table(&database_path, &table)
            .expect("create persistent database");

        println!();
        println!("created persistent database");
    }

    // Open persistent database.
    let mut disk = PersistentTable::open(&database_path).expect("open persistent table");

    println!();
    println!("=== Experiment ===");

    // Baseline: sequential scan.
    let before = benchmark_persistent_scan(&mut disk, "BD", 3).expect("benchmark scan");

    // Candidate: persistent index lookup.
    let after = benchmark_persistent_index(&mut disk, "BD", 3).expect("benchmark index");

    let lb = before.elapsed_ns as f64 / 3.0 / 1_000_000.0;
    let la = after.elapsed_ns as f64 / 3.0 / 1_000_000.0;

    let result = evaluate(
        &recommendation.target,
        BenchmarkMeasurement {
            latency_ms: lb,
            io_units: before.pages_read as f64,
        },
        BenchmarkMeasurement {
            latency_ms: la,
            io_units: after.pages_read as f64,
        },
    );

    println!("pages: {}", disk.page_count());
    println!("index entries: {}", disk.index_entry_count());

    println!(
        "baseline: {:.3} ms/query, {:.1} pages/query",
        lb,
        before.avg_page_reads()
    );

    println!(
        "indexed:  {:.3} ms/query, {:.1} pages/query, {} cache hits, {} misses",
        la,
        after.avg_page_reads(),
        after.cache_hits,
        after.cache_misses
    );

    println!(
        "baseline metrics: operation={}, rows={}, bytes/query={:.1}, evictions={}",
        before.operation,
        before.rows,
        before.avg_bytes_read(),
        before.evictions
    );

    println!(
        "indexed metrics: operation={}, rows={}, bytes/query={:.1}, evictions={}",
        after.operation,
        after.rows,
        after.avg_bytes_read(),
        after.evictions
    );

    println!(
        "latency improvement: {:.2}%",
        result.latency_improvement_pct
    );

    println!("page-read reduction: {:.2}%", result.io_reduction_pct);

    // Final authorization is based on measured experiment results.
    let result_allowed =
        crate::optimizer::validate_result(result.latency_improvement_pct, result.io_reduction_pct);

    println!(
        "result gate: {}",
        if result_allowed { "PASS" } else { "REJECT" }
    );

    // Validate persistent index after a fresh reopen.
    drop(disk);

    let (rows, restart_reads, restart_misses) =
        benchmark_restart_index(&database_path, "BD").expect("restart index");

    println!();
    println!("=== Restart Validation ===");
    println!(
        "restart index lookup: {} rows, {} physical page reads, {} cache misses",
        rows, restart_reads, restart_misses
    );

    println!("restart full-scan avoided: {}", restart_reads < 100_000);

    let accepted = result.accepted && result_allowed;

    println!();
    println!("=== Decision ===");
    println!("decision: {}", if accepted { "ACCEPT" } else { "REJECT" });
    println!("reward: {:.4}", result.reward);

    // Persist complete experiment history.
    let experiment_record = ExperimentRecord {
        target: recommendation.target.clone(),
        action: recommendation.action.clone(),
        accepted,
        latency_improvement_pct: result.latency_improvement_pct,
        io_reduction_pct: result.io_reduction_pct,
        reward: result.reward,
        timestamp: Utc::now().to_string(),
    };

    experiment_history.record(experiment_record);

    experiment_history
        .save(&history_path)
        .expect("save experiment history");

    // Feed measured experiment outcome back into the model.
    let feedback = Feedback {
        recommendation_target: recommendation.target.clone(),
        latency_before_ms: lb,
        latency_after_ms: la,
        accepted,
    };

    apply_feedback(&mut model_state, &feedback);

    model_store.save(&model_state).expect("save model state");

    println!();
    println!("=== Learning Feedback ===");
    println!("target: {}", feedback.recommendation_target);
    println!("accepted: {}", feedback.accepted);
    println!(
        "successful experiments: {}",
        model_state.successful_experiments
    );
    println!("failed experiments: {}", model_state.failed_experiments);
    println!("confidence boost: {:.4}", model_state.confidence_boost);

    println!();
    println!("=== Experiment History ===");
    println!(
        "total experiments: {}",
        experiment_history.experiments.len()
    );
    println!("history saved: {}", history_path.display());

    println!();
    println!("persistent state retained:");
    println!("  {}", database_path.display());
    println!("  {}", telemetry_path.display());
    println!("  {}", model_path.display());
    println!("  {}", history_path.display());
}
