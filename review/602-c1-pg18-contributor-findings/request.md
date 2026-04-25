# PG18 Contributor Diagnostic Findings

## Summary

This packet covers commit `4e4a9dfa6d9c9e968742af128b2700ad43995405`.

ADR-040 and Task 18 now record the current PG18 contributor result:

- the hidden contributor lifecycle behind one elected visible tuple emitter is
  valid under the strict PG18 serial-equivalence fixture;
- the current diagnostic path is not yet a performance improvement because
  non-elected workers publish duplicate initial graph cursors;
- blind bootstrap tail rotation or partitioning is not the next safe step,
  because the coordinator orders pending output by SQL score while serial HNSW
  rank can contain exact-score inversions.

## Result

The default elected-emitter lane still passes:

```text
Limit (actual time=13.540..14.488 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane also preserves serial equivalence, but it still
does not produce useful foreign handoffs:

```text
Limit (actual time=35.245..36.338 rows=16.00 loops=1)
next_runtime_blocker=PG18 diagnostic contributor env is enabled; non-emitting workers publish hidden coordinator output behind the elected visible tuple emitter
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: correctness is intact, but performance still needs a
rank-aware distinct contribution design. A naive tail-rotated worker could
publish a later exact-smaller candidate ahead of an earlier serial-rank row.

## Review focus

- Does the ADR wording correctly distinguish the validated hidden-slot
  lifecycle from the still-missing useful worker contribution?
- Does the Task 18 checklist point at the right next implementation slice:
  rank-aware distinct contribution behind one visible emitter?
- Is the warning against blind bootstrap partitioning concrete enough for the
  next coder to avoid reintroducing the old tail-rotation failure mode?

## Artifacts

- `artifacts/pg18-parallel-contributor-findings-default.log`
- `artifacts/pg18-parallel-contributor-findings-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-findings-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-findings-diagnostic.log`
