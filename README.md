# DreamizDB

AI-assisted adaptive database research prototype.

## v0.1 research target
Prove whether workload-aware prediction can improve physical database decisions without allowing AI to directly control execution.

Pipeline:
Query telemetry -> feature extraction -> prediction -> recommendation -> deterministic validation -> execution/benchmark -> feedback

## Implemented prototype modules
- Query fingerprinting
- Workload telemetry aggregation
- Adaptive index recommendation baseline
- Heat-to-tier mapping
- Recommendation confidence/cost model
- Deterministic recommendation validator

AI is advisory; the deterministic optimizer/policy layer remains authoritative.


## v0.3 Experimental Validation

Separates AI candidate recommendations from measured performance evidence. The current experiment API evaluates before/after latency and I/O and produces a reward signal. Benchmark inputs are currently supplied by the prototype; real storage benchmarking is the v0.4 target.

## v0.4.2

Storage module consolidated: `Record` and `Table` now live in `src/storage/mod.rs`.

## v0.5 Closed-Loop Adaptive Index

The executable now wires native storage, physical benchmarking, and experiment
evaluation into one deterministic closed-loop demonstration. The benchmark
compares a sequential scan against an actual in-memory BTree-backed lookup and
produces an ACCEPT/REJECT decision plus reward. This is still an in-memory
research prototype; it is not yet a persistent database engine.

## v0.5.1

Restored the benchmark module required by the library crate and closed-loop integration test.

## v0.6 Persistent Page Storage

Adds a native 4 KiB page store with persistent records, page-level country
indexing, explicit disk read/write counters, reopen/recovery validation, and
benchmarks based on actual page reads rather than result-row proxies.
The page format is intentionally simple and experimental; it is not yet a
transactional WAL-backed storage engine.

## v0.7 Buffer Pool + I/O Telemetry

Adds a bounded LRU buffer pool, cache hit/miss/eviction telemetry, and
per-query page/byte metrics. Persistent query execution now routes page reads
through the buffer pool. Benchmarks start cold so physical reads remain
measurable, while repeated accesses within the workload demonstrate cache
reuse.

## v0.8 Persistent Index + Restart Validation

Index metadata is stored beside the data file as a versioned `.idx` file.
Restarted databases load the country index without scanning every data page.
The index carries a SHA-256 table fingerprint and version/column metadata.
