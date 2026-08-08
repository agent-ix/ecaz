# Task 208: ec_distann NFR-021/NFR-022 Conformance Gates

Status: **complete — review-closed ACCEPT** (2026-08-08; feedback
`reviews/task-208/002-retrospective-sweep/feedback/2026-08-08-01-reviewer.md`).
Priority: P1 program integrity.

- Phases 1--3 implemented in `reviews/task-208/001-gates/` and **ACCEPTed**
  (`feedback/2026-07-30-01-reviewer.md`): the NFR-021 metric is normalized to
  bytes per owned graph record with a 100k/10k ratio `<= 2.0`, raw fixed-roster
  growth is emitted but no longer judged, missing evidence is `unavailable`
  rather than a pass, and pre-registration is validated before measurement.
  The former P2 about the NFR-021 head clause is resolved by Task 210's
  accepted membership-only, zero-byte-head gate.
- Phase 4, the arm-blind storage retrospective, is recorded in
  `reviews/task-208/002-retrospective-sweep/` and is review-closed ACCEPT.
  Task 210's accepted membership-only, zero-byte-head gate resolves the
  separate head-clause concern carried from packet 001.

Entry gate: Task 204's per-node storage emission, which this task consumes.

## Why

`NFR-021` and `NFR-022` (landed 2026-07-29) are currently prose. Task 203 showed
that prose gates do not hold: `NFR-017:38`, `NFR-018:36`, and `FR-078:492` each
already prohibited a replicated full index, and Tasks 198/199 cleared review
anyway because nothing mechanical checked them and no packet cited them.

The same failure mode is available again unless the invariant is machine-checked
on every run.

## Goal

Make NFR-021 and NFR-022 enforced by `ecaz bench suite` rather than by reviewer
attention.

## Phases

1. **NFR-021 resident-state gate.** Emit per-owner resident index bytes and
   owned-record counts per scale, compute the bytes-per-owned-record cross-scale
   growth ratio, and fail the run when normalized growth is superlinear or when
   any node holds non-owner graph records, non-owner full-precision vectors, or
   an unsharded O(N) derived relation. Raw fixed-roster bytes remain reported
   but are not a conformance threshold: a genuine O(N) shard necessarily grows
   with corpus cardinality when roster size is fixed. Cover derived, optional,
   and disabled-by-default relations — the FR-084 replica is the worked example
   of a relation that evaded every existing audit by not being literally a
   graph-node shard.
2. **NFR-022 arm labeling.** Label every arm's NFR-021 conformance in
   `results.jsonl` so an audit is mechanical. A run containing a non-conforming
   arm is permitted; a *decision* recorded against one is the failure, so the
   label must survive into the packet manifest.
3. **Pre-registration check.** Fail a suite run whose config declares a candidate
   without an NFR-021 admissibility verdict, so the screen happens before
   measurement rather than after.
4. **Retrospective sweep.** Re-read every committed distann packet for the
   arm-blind storage claim Task 203 identified ("storage identical/unchanged
   between arms") and re-classify those rows. This closes the `T4 = ?` cells in
   the Task 203 matrix and is the outstanding work item for
   `reviews/task-203/002-...`.

## Validation

Measurement/gate work, so the 10k/50k/100k closeout rule does not apply. Required
proof:

- the gate **fails** on a deliberately non-conforming arm (use the existing
  FR-084 replica as the negative fixture — it is a known violation with known
  bytes, 1,659,518,976 at 100k);
- the gate **passes** on the owner-traversal arm at 10k/50k/100k;
- the retrospective sweep output, as a table of packet -> claim -> re-classified
  status.

## Required review packets

1. `reviews/task-208/001-gates/` — implementation plus the pass/fail fixtures.
2. `reviews/task-208/002-retrospective-sweep/` — the re-classification table.

## Non-goals

- Changing NFR-021/NFR-022 semantics. If a gate cannot be implemented as
  specified, that is a finding to report, not a licence to weaken the NFR.
- Deleting or disabling the replica. It is useful here precisely as a negative
  fixture.

## References

- `NFR-021`, `NFR-022`, `NFR-018` (per-node term), `NFR-007`.
- `reviews/task-203/001-decision-reaudit/` Defects 4 and 4b.
