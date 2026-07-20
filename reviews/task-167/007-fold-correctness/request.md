# Task 167 packet 007 — multi-row fold correctness (167-006-P1)

Coder response to reviewer finding
`reviews/task-167/006-fold-greedy-descent/feedback/2026-07-09-01-reviewer.md`
**P1 — "Batch fold can select an unfolded delta row that has no directory
entry."** This packet closes that finding with a code fix + a regression that
reproduces the exact failure shape.

## The bug (as filed)

`graph_insert_record` reuses `collect_distann_hits` for the new node's candidate
search. That search merges the still-live delta chain into its result set
(`src/am/ec_distann/routine.rs:483-507`), so during a multi-row fold the hit set
for row A includes rows B, C … that have not yet been folded. Those rows have no
directory entry. If robust-prune selected one as a forward neighbor, the
mandatory forward-neighbor directory lookup
(`src/am/ec_distann/insert.rs:382-384`) errored — **after** earlier rows in the
same fold had already appended nodes, rewritten backlinks, and published a new
directory. The only prior fold pg_test folded a single row, so the multi-row
interaction was uncovered.

## The fix (commit `62590b3ea`)

Skip any candidate whose vec_id is absent from the current persisted directory
(`src/am/ec_distann/insert.rs`, candidate loop). Graph-placement candidates must
be persisted graph nodes; a not-yet-folded delta row is not part of the graph
and earns its own edges only when the fold loop reaches it, against the
then-current graph. This is exactly the reviewer's prescribed remedy ("Exclude
non-directory delta hits from graph-placement candidates").

## Regression (commit `62590b3ea`)

`test_ec_distann_fold_multi_row_clustered_delta` inserts three mutually-near
delta rows (coords 0.51/0.52/0.53 — so each is genuinely the other's nearest
neighbor, the shape that surfaces the bug) and folds all three in one
`ec_distann_fold_delta_into_graph` call. It asserts:

- all three fold without error (pre-fix: errored mid-fold on the directory
  lookup),
- `folded == 3`, the delta buffer drains, `node_count` grows by three,
- the whole folded cluster is graph-reachable (all of {90,91,92} in the top-3
  for the cluster centre; top-1 is not asserted because the near-identical
  coords are below the quantized `<#>` resolution).

## Validation

`cargo pgrx test pg18 --no-default-features --features pg18 multi_row_clustered`
→ `test result: ok. 1 passed`. Transcript:
`artifacts/multi-row-fold-test.log`. The pre-existing single-row
`test_ec_distann_fold_delta_into_graph` still passes.

## Scope note

This packet closes the 006 **P1 batch-fold-candidate** finding only. The other
006 findings (P0 distributed `aminsert`, P1 fail-before-publish dangling
backlinks, P1 FR-083-AC-4 suite evidence, P2 superlinear fold work) remain open
and are tracked for subsequent packets.
