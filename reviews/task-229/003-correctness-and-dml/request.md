---
task: 229
packet: 003-correctness-and-dml
agent: Codex
role: coder
model: gpt-5
date: 2026-08-27
seq: 01
---

# Task 229 correctness, DML, and runner review

Review source head `8b4618ca506a041693d3526b7f17aef7ef801393`
against exact current-main merge parent
`71004a18ce770f0c17501bd3d9942742d700a6ba`.

This request closes the packet gap identified in
`reviews/task-229/002-format-and-lifecycle/feedback/2026-08-27-07-reviewer.md`.
It does not ask the reviewer to re-review packet 002's accepted codec,
identity, catalog, or five-relation lifecycle work. The source slices in scope
are:

- `6e6a150df` — covered insert/replacement/delete atomicity;
- `c87dd7592`, `f75d9041e`, `72782f60f` — local/remote sidecar selection,
  visibility parity, fallback, corruption, and semantic tests;
- `2fe8cdbf3`, `79e3f8cb7` — feature-gated read A/B, owner/local telemetry,
  sidecar topology/storage accounting, DML gates, and structured suite output;
- `4c060ef9c` — the checked-in counterbalanced 10k/50k/100k suite; and
- `8b4618ca5` — explicit `payload_sidecar_live_content_digest` naming and the
  pre-DML-only initial/live equality assertion requested by seq-07.

Reviewer feedback commits `ec83fd671` and `49a4bf1ae` are present in the
branch history but contain no coder source and are not part of this source
review.

## Read-path contract implemented

- Task 222's typed exact attribute mask selects the sidecar only when every
  required physical attnum is covered. Whole-row, uncovered, unsupported,
  schema-mismatched, and legacy no-cover paths stay on the row tier.
- Remote owners perform one ordered TID-keyed batch lookup. A sidecar miss
  probes the exact row-tier TID under the same snapshot: both invisible keeps
  the existing remote skip; visible row tier plus missing sidecar is
  corruption and errors.
- Local Frozen rows are batched before executor consumption, retry misses
  under the same latest-snapshot path as the control, and preserve the
  existing `EC_GENERATION_MISSING` outcome when both attempts lose the exact
  row-tier version.
- Returned TID/`vec_id`, payload shape, descriptor/schema binding, and
  sidecar-relation presence are revalidated before exposing values. No partial
  sidecar/row-tier reconstruction is permitted.
- Production behavior defaults to sidecar-on only for a generation that
  explicitly declares a cover. The `benchmark_covering_sidecar` GUC and added
  detailed attribution fields are feature-gated under
  `distann-head-attribution-benchmark`; ordinary semantic result fields and
  the remote row shape are unchanged.

## DML and lifecycle behavior implemented

- Initial handoff and every Task 167 insert or same-identity replacement append
  a sidecar row keyed by the new row-tier TID in the same transaction as the
  row-tier/graph mutation. Replacement retains the old version and appends a
  new version; delete uses the existing graph tombstone/retention rule without
  a sidecar-specific delete.
- Sidecar payload encode failure, insert failure, and injected physical-DML
  failure roll back graph, row tier, and sidecar together. Covered publication,
  restart, retained-predecessor reads, abort, retire, and reclaim continue to
  use the cataloged five-relation generation.
- Topology now calls its recomputed relation digest
  `payload_sidecar_live_content_digest`. It equals the immutable Ready receipt
  `initial_content_digest` before DML only and is expected to diverge after
  legal post-Ready writes.

## Benchmark prerequisite implemented

- `covering_payload_attnums`, same-generation sidecar off/on variants,
  isolated-pair validation, and counterbalance position are suite-driven.
- The fixture records explicit sidecar selection in latency/stage/work rows,
  expects all 40 stage and 52 work rows, includes sidecar bytes in per-owner
  and generation storage totals, and accepts all 10 insert-work counters.
- Task 229 DML mode retains the single-index control for insert throughput,
  measures authoritative pre/post owner-topology row/byte deltas across remote
  owners, and records 32 routed single-row replacement and delete samples with
  mean/p50/p95/p99/max.
- `crates/ecaz-cli/suites/task229-covering-sidecar-10k-50k-100k.json` has 12
  fresh steps: `no-cover -> cover` and `cover -> no-cover` at each scale. Both
  covered builds run opposite-order same-generation read pairs. Every run dir
  is under `/home/peter/.ecaz/clusters`, fixture reuse is off, retention is
  unset, and compact artifact cleanup is enabled.

## Validation state

Static/shared-target preflight is green: `cargo fmt --check`, CLI test compile,
focused Task 229/covering CLI tests, full PG18+attribution check, and suite
audit (`12 steps`). Packet-local PG18 correctness/DML logs will be added to
this packet before its decision; packet 004 owns the release install and the
single authorized full-scale matrix. No custom Cargo target, new worktree,
database, fixture, or corpus was created for this request.

## Review questions

1. Do local and remote covered reads preserve the control's distinct snapshot,
   skip, retry, and fail-closed outcomes without partial reconstruction?
2. Are insert, replacement, delete, rollback, restart, and retained-generation
   semantics atomic and version-exact for the sidecar?
3. Does the runner now expose every preregistered read/storage/build/DML field,
   including replacement/delete p95 and remote-owner insert topology deltas,
   without comparing fresh and reused fixtures?
4. Does the live digest rename close seq-07's initial-vs-live naming trap, and
   may packet-local PG18 validation and packet 004 execution proceed without
   additional source changes?
