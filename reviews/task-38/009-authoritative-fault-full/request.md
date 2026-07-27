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
