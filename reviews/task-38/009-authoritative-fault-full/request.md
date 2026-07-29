---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
task: 38
packet: 009-authoritative-fault-full
status: review-requested
---

# Review Request: Task 38 Authoritative Fault Aggregate

Review code checkpoint `addeb885a` and the packet-local evidence described in
`artifacts/manifest.md`.

## Scope

This checkpoint replaces the former generic Make dependency list with one
live-only `ecaz dev fault full` orchestrator and a separate host-independent
`make fault-full-plan` surface.

The ordered aggregate contains 116 unique cases:

- 35 local smoke cases: five non-provider lanes across seven fixtures;
- 14 mutation-control cases across the same fixtures;
- 56 exact-path provider cases: heap/index EIO, ENOSPC, and slow disk plus WAL
  and temp ENOSPC for each fixture;
- four real remote-socket cases: DistANN TCP reset/slow and SPIRE named-Unix
  reset/slow; and
- seven systemd-scoped cgroup-v2 OOM cases.

Live mode preflights Linux, the built LD_PRELOAD provider, cgroup v2, a user
systemd manager, PG18 installation, disjoint roots, and empty evidence/runtime
directories. It sequences provider-off preparation and same-run slow
baselines, provider restart, post-restart arm-file creation, exact marker
oracles, disarm, restore, and shared postconditions. Remote cases reuse the
approved DistANN/SPIRE operators, and cgroup cases reuse the approved
seven-fixture operator with separately configurable ports.

Finalization now runs even after an execution failure: it captures the main
postmaster log delta, recursively audits packet-local `.log` files for
`PANIC:`, and runs shared cleanup postconditions while preserving all execution,
log-audit, and cleanup failures.

During implementation audit, the existing slow-disk provider was found to
sleep globally and emit no path-specific fault event. The checkpoint changes
it to delay only matching paths/file descriptors and emit the same exact
`fault=1 mode=slow-disk target=...` evidence required by the aggregate. A
Linux-gated regression test proves matched versus unmatched behavior when run
on the designated host.

## M5 Evidence

- `artifacts/fault-full-plan.log` and `artifacts/plan-counts.log`: the exact
  committed arm64 binary prints 116 unique cases with phase counts
  `35/14/56/4/7`.
- `artifacts/m5-live-preflight.log`: live mode lists the same matrix, then
  rejects macOS before creating either requested execution root.
- `artifacts/m5-build.log`: exact-checkpoint native build passes; the only
  warning is the existing unused `path` field in `corpus/load.rs`.
- `artifacts/fault-model-tests.log`: 9 M5-applicable fault-model tests pass.
  Linux-only LD_PRELOAD tests are target-gated and are not claimed here.

## Requested Review

Please verify:

- the 116-case plan covers every required fixture and exact provider/socket/
  cgroup surface without duplicates;
- live execution uses armed-after-restart, disarm-before-restore provider
  sequencing and same-run slow baselines;
- slow disk now honors exact path matching and emits an auditable fault event;
- every failure path still reaches log capture and shared cleanup finalization;
- child operator arguments, port allocation, and evidence/runtime roots remain
  isolated; and
- the packet does not overclaim Linux or Intel execution.

No AWS, remote host, CI, nightly, Docker, or Intel command was run. Task 38
must remain open after source approval until the full live aggregate passes on
the designated Intel/Linux host.

## Review Response: Sequence 1

Code checkpoint `c29c6dca5` closes the sole blocking finding in
`feedback/2026-07-26-01-reviewer.md`:

- the slow-disk oracle now computes
  `baseline_ms.checked_add(configured_latency_ms)` and reports a clear
  threshold-overflow error;
- provider time must be at least that exact threshold, rather than merely
  greater than the baseline;
- the success timing marker now includes `required_ms` and the comparison
  contract `provider-at-least-baseline-plus-configured-latency`; and
- a focused unit test proves equality passes, one millisecond below the
  threshold fails, and checked-add overflow fails.

The exact-checkpoint focused test result is stored in
`artifacts/slow-disk-threshold-test.log`. The live fourteen-case provider
execution remains designated Intel/Linux work and is not claimed by this
response.

## Review Response: 2026-07-27 Sequence 1

Code checkpoints `147d44d05` and `a35d1cd71` address every finding in
`feedback/2026-07-27-01-reviewer.md` and close the Apple-M5 scope:

1. Every live postcondition gate now captures an explicit `pg_stat_io` /
   `pg_stat_wal` baseline. A missing baseline is reported as
   `baseline_absent`, while `unavailable` remains reserved for a query that
   proves the view is unavailable.
2. `make fault-full` now executes the 35 local plus 14 mutation cases on
   macOS, emits three phase-specific Linux-only skip markers, and closes with
   `live_authority=partial executed=49 skipped=67`.
3. The LD_PRELOAD provider preserves `errno` across enable checks, fd matching,
   and fault-event recording.
4. The Linux-gated slow-disk regression uses a 500 ms delay and checks matched
   and unmatched wall-clock behavior.
5. Slow-disk baselines are measured after a provider-off restart, matching the
   provider-on measurement's cold-start shape.
6. All phase counts are derived from the case plan.
7. The no-panic audit uses lossy reads, accumulates failures, and writes its
   audit result even when a log cannot be read cleanly.
8. SPIRE sockets now remain below the supplied aggregate runtime root.

The repeated operator surface also accepts Make parameters, so local runs no
longer need approval-sensitive leading environment assignments:

```text
make fault-full FAULT_DATABASE=ecaz_fault_task38 \
  FAULT_HOST=/Users/peter/.pgrx FAULT_PORT=28818
```

The live M5 run exposed and then verified a separate DiskANN materialization
defect. Physical pages can contain unused line pointers, and PostgreSQL's
`PageAddItem` space accounting differs from the synthetic chain's conservative
capacity estimate. Checkpoint `a35d1cd71` preserves physical block/offset
addresses, skips unused slots during graph iteration, and materializes
already-accepted tuples with PostgreSQL-equivalent accounting.

Packet-local evidence includes:

- `artifacts/m5-mutation-control-postfindings.log`: all seven fixtures pass
  both mutation controls with real pgstat baselines;
- `artifacts/m5-partial-live.log`: the authoritative M5 aggregate passes
  `49/49` executable cases and explicitly skips `67` Linux-only cases;
- `artifacts/m5-partial-live/main-postmaster.log` and
  `artifacts/m5-partial-live/no-panic-audit.log`: crash-recovery and no-panic
  evidence; and
- `artifacts/diskann-physical-page-materialization-test.log`: both focused
  physical-page tests pass.

No AWS, remote host, CI, nightly, Docker, or Intel command was run. Task 38
remains open for the 67 provider/remote-socket/cgroup cases on the designated
Intel/Linux host; the Apple-M5 scope is complete.

## Review Response: 2026-07-28 Sequence 2

Findings 9 and 10 in `feedback/2026-07-28-02-reviewer.md` are addressed by
checkpoints `2100e7310` and `50b7690d8`.

- The OOM lane now marks its AM SQL and, immediately before SIGKILL, requires
  `pg_stat_activity` to show that PID actively executing the marked workload.
  A completed workload parked in the safety hold is rejected.
- DiskANN physical TID divergence and missing-page states are returned errors
  in release builds; unused slots have explicit state and cannot be returned as
  successful empty tuples.
- A PG18 regression creates and scans across a real unused line pointer.

The production DiskANN change and its required release-backend A/B evidence are
isolated in the new review packet
`../010-diskann-physical-page-materialization/`. Its checked-in
`ecaz bench suite` configuration completed baseline/candidate recall, latency,
and storage at 10k, 50k, and 100k. Recall and DiskANN storage are identical at
every measured point, with no systematic latency regression.

No AWS, remote host, CI, nightly, Docker, or Intel command was run. The M5
findings are ready for outside re-review; Task 38 remains open for the 67
Intel/Linux cases.
