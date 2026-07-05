# Task 141: SPIRE Multinode Bench Integrity — Release Substrate + Guard Loophole

Status: proposed (2026-07-04; filed from the SPIRE latency root-cause
investigation, operator-approved remediation program, task 1 of 6).
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — blocks every latency claim in Tasks 142–146. Nothing
downstream is measurable until this lands.

## Why

Every multi-instance SPIRE latency number ever produced is debug-build.
The `spire-local-multinode` fixture hard-codes
`cargo pgrx install --test --features "pg18 pg_test" --no-default-features`
(`crates/ecaz-cli/src/commands/dev/spire_multicluster.rs:1171-1187`), which
installs the dev-profile `.so` (opt-level 0, debug_assertions on, `pg_test`
feature) on all four nodes. The `ecaz bench suite` release preflight
(`crates/ecaz-cli/src/commands/bench/suite.rs:875`,
`manifest_has_release_guarded_steps` at `:3934`) only guards `latency` and
`recall` step kinds; the multinode cells use `storage` + `spire-pipeline`
steps and slipped through. Taints: Task 123 packets 009–021, Task 131
multi-instance latencies, the entire Task 139 phase-1 grid (127–1600 ms p50
at 50k). Calibration: task-123 packet 009 measured the same substrate at
87.3 ms p50 @ 100k recall 1.0; release single-instance baselines on this host
are IVF 100k 0.9980@37.7 ms, SPIRE-local 100k 0.9975@147 ms.

## Goal

A multinode fixture that installs and verifies a release `ecaz.so`; a suite
runner with no unguarded latency-claiming step kinds; a release-build anchor
re-baseline that quantifies the debug distortion and gives the program its
first honest distributed numbers.

## Scope

### Phase 0 — Fixture + guard fix

- `spire-local-multinode` installs the release-profile extension (keep a
  `--debug` escape hatch for pg_test-dependent drills, clearly labeled).
- Extend the release preflight to `spire-pipeline` (and any step kind that
  emits latency fields); record `SELECT ecaz_build_profile()` per node in
  every `suite-manifest.json` (`backend` must never be null for latency
  claims).
- Audit for other loopholes: any step kind emitting latency without the
  guard.

### Phase 1 — Anchor re-baseline (release)

- Re-run a small anchor set from the Task 139 grid on the fixed fixture:
  50k n128/b0, 50k n1024/b0, one 100k cell (n1024/b0), standard sweep,
  200 queries, `source_identity=include`.
- Report the debug→release distortion factor per phase (routing vs scan vs
  transport) and re-anchor the corpus-fraction-scanned/recall columns
  (expected unchanged) vs latency columns (expected large shifts).

### Phase 2 — Reconcile the 87 ms discrepancy

- Task-123 packet 009 (100k n1024/b2 nprobe64 = 87.3 ms p50, 32 queries)
  vs Task 139 grid (~600 ms at 50k): decompose query count, identity-on,
  config drift, build profile. Publish the reconciliation so no stale number
  survives unexplained.

## Required Evidence

- `ecaz bench suite` runs with per-node build-profile probes in manifests;
  packet-local logs; A/B debug-vs-release on at least one anchor cell to
  document the distortion factor.

## Non-Goals

- No routing/scan behavior changes (Tasks 142–145).
- No full grid re-run (Task 146).

## Acceptance Criteria

1. Multinode fixture installs release `.so`; manifests record per-node build
   profile; preflight rejects debug backends for all latency-claiming steps.
2. Anchor cells re-measured on release with distortion factors published.
3. 87 ms-vs-663 ms discrepancy reconciled in the packet.
4. A taint annotation exists in the Task 139 packet 001 manifest (coder
   wind-down item; verify done).

## References

- `reviews/task-139/001-phase1-nlists-boundary-grid/feedback/2026-07-04-01-agent-ix.md` (finding + wind-down)
- `crates/ecaz-cli/src/commands/dev/spire_multicluster.rs:1171-1187`
- `crates/ecaz-cli/src/commands/bench/suite.rs:875,3934`
- `reviews/task-123/009-multi-instance-phase-a-baseline/request.md`
- `benchmarks/task76-intel-local-spire-pareto/manifest.md` (release targets)
