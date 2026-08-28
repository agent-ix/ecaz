# Task 229: ec_distann Covering Payload Sidecar

Status: **implementation and evidence complete — packets 001--003
review-closed ACCEPT/DONE; packet 004 full-scale decision review-open with
STOP. The completed suite proves every 40-stage/52-work-row telemetry cell and
the DML distributions. One disclosed limitation remains: the coordinator
remote-row disappearance branch is statically reviewed but not dynamically
injected** (updated 2026-08-28; latest closed review
`reviews/task-229/002-format-and-lifecycle/feedback/2026-08-28-11-reviewer.md`;
decision request `reviews/task-229/004-full-scale-decision/request.md`).
Priority: P0 storage/retrieval latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
MAT-27 and ARCH-06. This is the first of four isolated retrieval-layout tasks;
Tasks 230--232 must still run even if this candidate wins.

## Why

The production lazy-10 100k control materializes 6.66 remote result rows per
scan, but the id-only query currently requests four attributes, returns
123,076.8 payload bytes, and spends 8.752 ms/scan in owner payload SQL. Task
222 removes attributes that are not required. It does not eliminate the row
tier lookup, tuple deformation, detoast/send machinery, or remote
materialization round for attributes that are required.

A small generation-owned covering sidecar can store a declared set of common
scalar result attributes in their already-canonical binary form. When the
proven Task-222 attribute mask is a subset of the cover, the owner can answer
without opening the full row tier. Wider and ambiguous queries fall back to
the normal row tier.

## Goal

Implement and benchmark an opt-in, generation-scoped covering sidecar for a
small, explicitly declared scalar projection. Prove whether avoiding the full
row-tier path improves retrieval enough to justify its storage, lifecycle, and
DML cost.

## Entry conditions

1. Task 222 is review-closed and supplies the fail-closed attribute-use mask.
2. Tasks 223 and 224 have reported their direct-access and locality findings,
   so the baseline owner cost is decomposed rather than inferred.
3. The control is frozen at the post-224 production disposition. Later layout
   tasks compare against this same unstacked control, even if this task wins.
4. Task 239 reproduces and diagnoses the current native 12/10 bounded-read
   divergence before this task claims semantic closeout. **Satisfied:** Task
   239 review-closed ACCEPT identifies a shared-session harness GUC leak and
   restores exact-main production lazy-10 to 6 remote + 4 local = 10 reads for
   10 rows without widening the bound.

## Required implementation

### P1 — Format and selection contract

- Add an opt-in generation format that records the exact covered attnums,
  row-schema fingerprint, binary I/O identity, null representation, and sidecar
  digest.
- Persist one compact entry per owned vec_id containing only the selected
  scalar attributes. Choose and document one bounded lookup representation;
  do not build multiple sidecar variants in the same task.
- Use Task 222's attribute mask to select the sidecar only when every required
  attribute is covered. Whole-row Vars, unsupported types, schema mismatch, or
  uncovered quals fall back to the row tier without partial reconstruction.
- Keep the sidecar owner-local. No O(N) coordinator replica or cache is allowed.

### P2 — Lifecycle and DML

- Bind sidecar bytes and schema identity into the generation descriptor,
  receipt, manifest, and epoch fingerprint.
- Build, handoff, publish, retain, retire, reclaim, restart, and rollback the
  sidecar with its owning generation.
- Task 167 inserts and replacements update the sidecar atomically with the
  graph/row-tier mutation; deletes obey the existing tombstone/retention rule.
- Old generations and indexes without the sidecar remain readable through the
  existing row-tier path.

### P3 — Evidence

- Exercise id-only, covered multi-scalar, uncovered scalar, qual-only,
  `SELECT *`, NULL, TOAST, mixed-owner, deepening, rescan, insert, replacement,
  delete, restart, and outage cases.
- Run isolated sidecar-off/on A/B cells at 10k, 50k, and 100k through a checked-in
  `ecaz bench suite` config. The arms must otherwise share projection policy,
  generation inputs, search settings, and release provenance.
- Compare arms at matched fixture position, or use a preregistered
  counterbalanced envelope that separates position/warmth from the candidate.
  Never compare a fresh-build control only against a reused candidate.
- Report result/recall identity, mean/p50/p95/p99/max, owner stages, heap and
  sidecar reads, bytes by attribute, wire bytes, build time, DML work, per-node
  storage, and NFR-021/NFR-022 conformance.

## Decision rule

The task cannot close on design analysis or a 100k screen alone. It closes only
after the prototype and full 10k/50k/100k matrix establish PROMOTE or STOP.
A win authorizes a separate production-default disposition; it does not remove
Tasks 230--232 from the operator-selected comparison.

## Non-goals

- Covering vectors, arbitrary large payloads, or every source attribute.
- Replacing the graph-store layout or full row tier.
- Adding an unbounded payload cache or coordinator copy.
- Stacking Task 230, 231, or 232 storage mechanisms into this arm.

## Acceptance

1. The sidecar format is versioned, independently decoded, corruption-tested,
   and tied to the exact generation and row schema.
2. Sidecar selection is fail-closed and all fallback queries are byte-identical
   to the control.
3. DML, publication, recovery, retention, and reclaim tests pass on PG18.
4. Packet-local 10k/50k/100k recall, latency, storage, and build/DML evidence
   supports an explicit PROMOTE or STOP decision.

## Required review packets

1. `reviews/task-229/001-plan/`
2. `reviews/task-229/002-format-and-lifecycle/`
3. `reviews/task-229/003-correctness-and-dml/`
4. `reviews/task-229/004-full-scale-decision/`

## References

- Tasks 222--224 and Task 239
- Task 218 production lazy-10 attribution
- Roadmap MAT-27 / ARCH-06
- FR-076, FR-079, FR-082, FR-083, NFR-016, NFR-018, NFR-021, NFR-022
