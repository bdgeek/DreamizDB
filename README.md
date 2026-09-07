# DreamizDB

**AI-assisted adaptive database research prototype written in Rust.**

DreamizDB explores whether workload-aware and AI-assisted intelligence can improve database physical decisions while keeping actual planning, validation, and execution deterministic, measurable, and safe.

> **Research principle:** AI may recommend; deterministic database policy, cost models, validation, and execution remain authoritative.

---

## Research Goal

DreamizDB investigates a closed-loop adaptive database architecture:

```text
Query Workload
      ↓
Telemetry
      ↓
Feature Extraction
      ↓
Prediction / Recommendation
      ↓
Deterministic Cost & Policy Validation
      ↓
Query Planning
      ↓
Query Execution
      ↓
Physical Benchmarking
      ↓
Experiment Evaluation
      ↓
Feedback
```

The project is an experimental research platform. It is intended to validate database optimization ideas incrementally rather than immediately compete with production database engines.

---

## Current Milestone

### Cost-Based Query Planning & Execution

The current implementation provides a working SQL query parsing, planning, cost-selection, and execution pipeline over persistent storage.

### Implemented

- SQL `SELECT` query parsing
- Multiple-column projection
- `WHERE` predicates
- `AND` / `OR` boolean expressions
- Nested boolean expressions
- Comparison operators:
  - `=`
  - `!=`
  - `<`
  - `<=`
  - `>`
  - `>=`
- Sequential table scans
- Persistent country-index lookups
- Query plan representation
- Cost-based index selection
- Persistent query statistics
- Match-set-aware planning
- Query execution against persistent storage
- Projection during execution
- Persistent index restart validation
- Buffer pool and I/O telemetry
- Physical page-read benchmarking
- Experiment evaluation and reward calculation
- AI-assisted adaptive recommendation pipeline

---

## Cost-Based Query Planner

DreamizDB does not blindly use an index whenever one exists.

For an indexable predicate, the planner compares estimated costs:

```text
Sequential Scan Cost ≈ total rows

Indexed Lookup Cost ≈ index lookup + matching rows
```

For example:

```text
1,000 rows
10 matching rows
        ↓
Index lookup preferred
```

while:

```text
1,000 rows
990 matching rows
        ↓
Sequential scan preferred
```

This allows the optimizer to make a deterministic decision based on persistent table statistics.

The current cost model is intentionally simple and research-oriented. It provides a foundation for progressively more realistic cardinality and physical-cost models.

---

## Query Execution

The query engine currently supports queries such as:

```sql
SELECT *
FROM users;
```

```sql
SELECT *
FROM users
WHERE country = 'BD';
```

```sql
SELECT id, country
FROM users
WHERE country = 'BD';
```

```sql
SELECT *
FROM users
WHERE country = 'BD'
  AND value > 100;
```

```sql
SELECT *
FROM users
WHERE country = 'BD'
   OR country = 'US';
```

```sql
SELECT *
FROM users
WHERE country = 'BD'
  AND (value >= 100 OR value = 400);
```

The planner can choose between:

```text
SequentialScan
```

and:

```text
IndexedLookup
```

based on query structure, index availability, and persistent statistics.

---

## Storage Architecture

DreamizDB currently contains:

- Persistent page storage
- Native 4 KiB pages
- Persistent records
- Country index
- Versioned `.idx` index metadata
- SHA-256 table fingerprinting
- Restart/recovery validation
- Explicit disk read/write accounting
- LRU buffer pool
- Cache hit/miss/eviction telemetry
- Per-query page and byte metrics

The storage layer remains intentionally experimental and is **not yet a transactional WAL-backed production database engine**.

---

## Adaptive / AI Layer

The AI layer is deliberately separated from database execution.

Current research components include:

- Query fingerprinting
- Workload telemetry aggregation
- Adaptive index recommendation
- Heat-to-tier mapping
- Recommendation confidence/cost modeling
- Deterministic recommendation validation
- Experiment history
- Model state persistence
- Closed-loop learning from measured results

The architecture follows:

```text
AI Recommendation
       ↓
Deterministic Validation
       ↓
Cost Model
       ↓
Query Planner
       ↓
Execution
```

AI therefore cannot directly issue arbitrary physical database operations.

---

## Experimental Validation

The project includes tests covering:

- Query parsing
- Query planning
- Cost-based planning
- Persistent cost-based planning
- Query execution
- Persistent storage
- Persistent index restart
- Buffer pool behavior
- Adaptive workload analysis
- Closed-loop learning
- Telemetry
- Pipeline behavior

Latest local validation:

```text
cargo check --lib        ✓
cargo check --bin        ✓
cargo test               ✓

61 tests passed
0 tests failed
```

The test suite is intended to protect deterministic planner behavior while the storage, optimizer, and adaptive layers continue to evolve.

---

## Project Structure

```text
DreamizDB/
├── src/
│   ├── ai.rs
│   ├── benchmark.rs
│   ├── experiment.rs
│   ├── features.rs
│   ├── optimizer.rs
│   ├── query/
│   │   ├── executor.rs
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── planner.rs
│   │   └── types.rs
│   ├── statistics.rs
│   ├── storage/
│   │   ├── buffer.rs
│   │   ├── index.rs
│   │   ├── persistence.rs
│   │   └── mod.rs
│   └── telemetry.rs
│
├── tests/
│   ├── adaptive_workload.rs
│   ├── buffer_pool.rs
│   ├── closed_loop.rs
│   ├── persistent_index_restart.rs
│   ├── persistent_storage.rs
│   ├── pipeline.rs
│   ├── query_cost_planner.rs
│   ├── query_cost_planner_persistent.rs
│   ├── query_executor.rs
│   ├── query_parser.rs
│   └── query_planner.rs
│
├── docs/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
└── SECURITY.md
```

---

## Build

### Prerequisites

- Rust toolchain
- Cargo

### Format

```bash
cargo fmt
```

### Check the library

```bash
cargo check --lib
```

### Check the executable

```bash
cargo check --bin dreamizdb
```

### Run the complete test suite

```bash
cargo test
```

---

## Development Philosophy

DreamizDB is being developed incrementally as a database research platform.

The project prioritizes:

1. Deterministic behavior
2. Measurable physical effects
3. Persistent state
4. Reproducible experiments
5. Explicit cost models
6. Safe AI recommendations
7. Test-driven validation

Each optimization should demonstrate measurable evidence before becoming authoritative.

---

## Milestone History

### v0.1 — Research Foundation

Established the core research target: determine whether workload-aware prediction can improve physical database decisions without allowing AI to directly control execution.

Initial pipeline:

```text
Query telemetry
→ feature extraction
→ prediction
→ recommendation
→ deterministic validation
→ execution / benchmark
→ feedback
```

### v0.3 — Experimental Validation

Introduced before/after experiment evaluation using latency and I/O measurements and a reward signal.

### v0.4.2 — Storage Consolidation

Consolidated the native storage module around `Record` and `Table`.

### v0.5 — Closed-Loop Adaptive Index

Connected native storage, physical benchmarking, and experiment evaluation into a deterministic closed-loop demonstration.

### v0.5.1 — Benchmark Integration

Restored the benchmark module required by the library and closed-loop integration test.

### v0.6 — Persistent Page Storage

Added:

- Native 4 KiB page storage
- Persistent records
- Page-level country indexing
- Disk read/write counters
- Reopen/recovery validation
- Physical page-read benchmarking

The page format remains intentionally simple and experimental.

### v0.7 — Buffer Pool + I/O Telemetry

Added:

- Bounded LRU buffer pool
- Cache hit/miss/eviction telemetry
- Per-query page/byte metrics
- Buffer-pool-backed persistent query reads
- Cold-start physical-read benchmarking

### v0.8 — Persistent Index + Restart Validation

Added:

- Versioned `.idx` index metadata
- Persistent country index
- Restart-time index loading
- SHA-256 table fingerprint
- Index version and column metadata
- Protection against using an index for modified database contents

### Current — Cost-Based Query Planning & Execution

Added:

- SQL query parser
- Query expression types
- Query planner
- Sequential scan planning
- Persistent indexed lookup planning
- Persistent query statistics
- Cost-based index-versus-scan selection
- Query executor
- Projection
- Boolean predicates
- Numeric comparisons
- Nested predicates
- Persistent query-planning tests
- End-to-end query execution tests

---

## Roadmap

### Completed

- [x] Query telemetry
- [x] Workload feature extraction
- [x] Adaptive index recommendation
- [x] Deterministic optimizer validation
- [x] Experiment evaluation
- [x] Persistent page storage
- [x] Buffer pool
- [x] Persistent index
- [x] Restart validation
- [x] SQL query parser
- [x] Query planner
- [x] Cost-based index selection
- [x] Persistent query statistics
- [x] Query executor
- [x] Projection
- [x] Boolean predicates
- [x] Numeric comparisons
- [x] End-to-end query tests

### Next Research Targets

- [ ] Multi-column indexes
- [ ] More general indexable predicates
- [ ] Improved cardinality/selectivity estimation
- [ ] More realistic physical cost models
- [ ] Query plan instrumentation
- [ ] Automatic index creation/removal experiments
- [ ] Larger workload simulations
- [ ] Concurrent query execution
- [ ] Transaction support
- [ ] WAL/recovery architecture
- [ ] More advanced adaptive/ML models
- [ ] Benchmark comparison against established database engines

---

## Research Status

DreamizDB is a **research prototype**, not a production-ready relational database.

The project is intentionally evolving through measurable milestones. Transactional durability, concurrency, recovery guarantees, broad SQL compatibility, sophisticated cardinality estimation, and production operational guarantees remain future work.

---

## License

Apache-2.0
