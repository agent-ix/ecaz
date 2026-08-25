# Task 227 clean integration validation manifest

- Head SHA under test: `e8df44f2978b4375ffed6dbb547ce12cff2ddbd6`.
- Task bucket / packet: `reviews/task-227/006-main-integration/`.
- Integration base: Task 226 clean evidence branch at
  `eba7a4e908d7728f16d8d15c524cfc9f2620a99e`; this branch is stacked only
  because Task 227's frozen control and diagnostic ceiling consume Task 226's
  disposition. It contains no Task 226 production-default change.
- Storage format and rerank mode: unchanged production format; Task 227 adds
  benchmark-feature-gated traces and read-only diagnostics. The frozen
  measurement used RaBitQ plus exact-neighbor diagnostic arms and exact final
  rerank, as recorded in packet 005.
- Fixture: no new cluster or corpus run was performed for clean integration.
  Immutable 100k three-owner physical-generation evidence, the separate
  monolithic control, query slice hashes, commands, execution SHAs, and
  one-generation attestation remain in
  `reviews/task-227/005-query-level-attribution/artifacts/manifest.md`.
- Timestamp: 2026-08-24 America/Los_Angeles.

## Clean-integration checks

- `diagnostic-replay-test.log`
  - Command: `cargo test -p ecaz-cli diagnostic_replays_match_the_shipped_sharded_head_default`
  - Result: 1 passed, 0 failed; exit 0.
- `reuse-provenance-tests.log`
  - Command: `cargo test -p ecaz-cli reuse_`
  - Result: 2 passed, 0 failed; exit 0.
- `query-trace-quality-bar-test.log`
  - Command: `cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark query_trace_preserves_rerank_input_when_final_results_are_truncated`
  - Result: 1 passed, 0 failed; exit 0.

No repository-wide formatter was run. No corpus, truth cache, PGDATA,
polling output, or operational exhaust was generated for this integration.
