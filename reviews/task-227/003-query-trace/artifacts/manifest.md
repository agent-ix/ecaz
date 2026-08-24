# Task 227 packet 003 artifact manifest

- Code SHA: `a9f66b120` (`Add bounded DistANN query traces`)
- Task bucket / packet: `reviews/task-227/003-query-trace/`
- Lane: PG18 feature-only diagnostic tooling; no benchmark measurement
- Timestamp: 2026-08-24 04:27 PDT (America/Los_Angeles)
- Fixture: focused Rust mock graph, pure CLI suite expansion, and one isolated
  one-index/two-row PG18 physical-generation callback fixture
- Storage format / rerank: RaBitQ physical generation; exact-expanded result
  ordering; no production rerank or scan-policy change

Artifacts:

- `query-trace-unit-tests.log`
  - Command: `cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark query_trace`
  - Result: 1 passed, 0 failed
  - Covers the 65,536-locator bound, truncation signal, >32-seed origin safety,
    operation isolation, and replica-attempt reset.
- `query-trace-round-tests.log`
  - Command: `cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark gateway_trace_records_shared_expansion_origins`
  - Result: 1 passed, 0 failed
  - Covers seed scores, requested/returned/exact ids, retained frontier,
    exact-ranked input, final ids, and termination state.
- `production-fast-path-test.log`
  - Command: `cargo test --lib --no-default-features --features pg18 distann_orchestration_expands_no_vec_id_twice_and_respects_cap`
  - Result: 1 passed, 0 failed
  - Confirms the feature-disabled production traversal remains correct.
- `query-trace-cli-tests.log`
  - Commands: `cargo test -p ecaz-cli distann_query`; `cargo test -p ecaz-cli distann_local_multinode_expands_staged_query_slice`
  - Result: 3 passed, 0 failed
  - Covers physical-only validation, exact query-slice prerequisites, command
    expansion, and expected trace-artifact declaration.
- `query-trace-pg18-callback.log`
  - Command: `cargo pgrx test pg18 test_distann_query_trace_callback`
  - Result: 1 passed, 0 failed (focused PG18 callback; unrelated targets
    filtered)
  - Covers a complete two-row physical build/publish, persisted head open,
    traversal, exact-result containment, generation identity, bounded JSONB
    return, and participant fingerprint addressing.

No corpus/query TSV, truth cache, PGDATA cluster, polling output, or benchmark
measurement is included. The PG18 fixture is transactional and isolated; no
shared-table or external cluster surface was used.
