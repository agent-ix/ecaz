# Task 210 P0 review request: the coordinator's own state is now measured

Commit under review: `f71fbcc90` (emitter + gate), with the requirement slice
`49751698c` (NFR-021) and the task-ownership slice `a32955471`.

## Why this exists

Task 203's audit began because ec_distann had stopped being sharded. The finding
reached `StR-008`, `NFR-021`, `NFR-022`, and the ledger, but it reached no task:
`TRAV-30` was ACTIVE with no owner, FR-084's disposition was a non-goal in six
tasks, head sharding was "optional" in Task 207 Phase 3, head replication was
untasked — and the coordinator storage row was hardcoded to zero, so the one
unsharded structure in the system was invisible to the gate built to catch
unsharded state. Task 210 owns all of it; this is its first phase.

## What changed

- `distann_multicluster.rs`: the coordinator `physical_benchmark_storage_node`
  row is measured and itemised per relation instead of emitted as
  `graph_bytes=0 directory_bytes=0 control_bytes=0`. Head sample and head graph
  are emitted as classified `physical_benchmark_storage_relation` rows, marked
  `arm_invariant=true` because they are generation-scoped — stated, not implied
  by reprinting a pre-loop scalar, which was the Task 204 defect.
- `suite.rs`: the NFR-021 derivation consumes those rows. A coordinator-resident
  unsharded relation is reported by name, bytes, and owning phase in
  `outstanding_distribution_gap` on every conformance row. One that is not on
  the known-gap list is a hard violation.
- `NFR-021`: distribution is now defined in five clauses, and the constant-`C`
  coordinator-resident head exemption is **removed** — a structure the reference
  design distributes is distributed here even when it is small (§2.2, §4.1).

## Result

At 100k the coordinator holds **25,894,607 bytes** of unsharded index state that
the gate previously reported as 0: 4,096 full-precision f32 landmarks
(25,280,512 B) plus their head graph (614,095 B). Constant in N at all three
scales — which is precisely why the removed exemption was load-bearing in the
wrong direction.

The owner arm is `conforming` with normalized growth 1.094675 across a 10×
corpus, so lanes that did not introduce the head gap are not halted; the gap is
attributed to Task 210 P2 on every row and clears when P2 lands.

Full numbers, provenance, and the re-run command are in
`artifacts/manifest.md`; structured rows in `artifacts/run/results.jsonl`.

## Validation

29 focused suite tests pass (24 before this task), clippy exit 0, and the
three-scale run above. This packet changes measurement only — no index
behaviour — so the 10k/50k/100k A/B closeout rule applies to P1–P3, not here.

## Reviewer questions

1. Is the known-gap mechanism the right shape? It reports honestly and loudly
   without failing unrelated lanes for a gap they did not introduce — but it is
   a deliberate choice not to hard-fail every distann suite until P2 lands.
2. Is `arm_invariant=true` sufficient labelling for the head rows, given Task
   204's finding about values that are measured once and printed per arm?

Request open.
