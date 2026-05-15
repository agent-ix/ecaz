# Review Request: SPIRE Phase 13d Read Efficiency and Observability

**Requester:** coder1
**Date:** 2026-05-15
**Code commit:** `d2e1334cf927d7e40a0365a71f453ca336039bdf`
**Review focus:** final production-read measurement and low-risk efficiency
fixes before AWS read workload execution.

## Summary

This slice lands the Phase 13d task and implements the efficiency review plan:

- adds
  `ec_spire_remote_search_production_read_profile(index_oid, query, top_k)` as
  an explicit live production-read profile surface returning `(metric, value)`
  rows for timing/count attribution;
- records planning, fingerprint guard, conninfo lookup, socket/TLS, statement
  timeout setup, remote regclass, endpoint identity, candidate receive, heap
  receive, payload decode, merge, strict failure, timeout/cancel, and degraded
  skip metrics;
- reuses one async libpq session per remote dispatch for candidate receive and
  heap receive, while keeping candidate and heap I/O overlapped across remotes;
- keeps default operator diagnostics cheaper by avoiding full live heap
  resolution unless the operator asks for the heap summary/profile surface;
- reduces full-sort work in candidate and heap merges by using partial
  selection before the final deterministic sort/truncate;
- extends the PG18 multicluster CustomScan smoke to assert the new profile
  counters on the live read path.

## Files To Review

- `plan/tasks/task30-phase13d-spire-read-efficiency-observability.md`
- `docs/SPIRE_DIAGNOSTICS.md`
- `src/lib.rs`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`
- `src/am/ec_spire/coordinator/remote_candidates/production_transport.rs`
- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs`
- `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
- `scripts/run_spire_multicluster_customscan_read_pg18.sh`

## Validation

- `cargo check --no-default-features --features pg18` passed. It still reports
  the pre-existing unused-import warning in `src/am/mod.rs`.
- `cargo fmt` passed with the repository's existing stable-rustfmt warnings for
  nightly-only formatting options.
- `git diff --check` passed.
- `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/766-c1-spire-phase13d-read-efficiency-observability/artifacts`
  passed. Key lines:
  `Custom Scan (EcSpireDistributedScan)`,
  `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`,
  `typed_payload_probe=ready,pg_binary_attr_v1,t,t`,
  `profile_summary=ready|remote_ready|1|1|1|1|1|1|1`.
- `cargo test production_read_profile_row_preserves_metric_rollup --lib --no-default-features --features pg18`
  and
  `cargo test remote_heap_candidate_result_merge_reports_duplicates_before_top_k --lib --no-default-features --features pg18`
  both built the test binary but could not execute under the plain lib harness
  because it exits with `undefined symbol: pg_re_throw`; no assertions ran.
  Packet-local logs capture the failures.

## Known Limits

- The new profile function is intentionally live: it opens remote connections
  and executes the production candidate/heap path. Routine triage should keep
  using the dry executor and pipeline surfaces unless live attribution is
  desired.
- The local smoke validates one remote dispatch and the single-session counter
  shape. It does not simulate cross-AZ RTT, Secrets Manager latency, EC2 CPU
  scheduling, or EBS stalls; AWS packets should cite the new profile rows beside
  latency logs for those effects.
- Direct plain `cargo test --lib` execution remains blocked by the local pgrx
  loader/symbol issue, so executable local coverage is through the installed
  PG18 multicluster smoke.
- The two unrelated dirty Python test files already present in the worktree are
  not part of this checkpoint.

## Reviewer Questions

1. Is the explicit `(metric, value)` profile surface acceptable for Phase 13d,
   given the pgrx tuple-width limit and likely future metric additions?
2. Is the session-reuse boundary correctly placed in the production
   candidate/heap adapter, or should session ownership move closer to the
   fanout executor?
3. Is the operator diagnostics split acceptable: cheap default diagnostics, with
   full heap execution moved behind explicit summary/profile calls?
