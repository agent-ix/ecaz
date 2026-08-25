# Task 222: ec_distann Qual-Aware Payload Projection

Status: **implementation and required 10k/50k/100k evidence complete; reviewer
seq-03 findings addressed; clean current-main packet 005 final-review-open**
(2026-08-24). Priority: P0 latency. Candidate retained:
the exact id-only mask preserves recall, ordered-result identity, and storage
while improving warm mean latency by 33.33%-40.41%. Plan revisions applied
against `reviews/task-222/001-plan/feedback/2026-08-23-01-reviewer.md`;
closeout requests: `reviews/task-222/004-full-scale-decision/request.md` and
`reviews/task-222/005-main-integration/request.md`.
Reviewer cleanup commit: `06b59c4c6`.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
MAT-28 and MAT-29. Origin: Task 218's production lazy-10 attribution and the
Task 220/221 negative owner-materialization screens.

## Why

The production lazy-10 100k profile materializes only 6.66 remote rows per
scan, yet requests four columns per row, returns 123,076.8 payload bytes per
scan, and spends 8.752 ms/scan in owner payload SQL. The standard latency SQL
projects only `id`, but its executor target list is expected also to contain
the `embedding <-> $q` ordering expression as a resjunk entry.
`build_payload_metadata` currently ships every non-dropped column because an
older target-list-only implementation omitted qual columns and evaluated
remote predicates against NULL.

Whole-row shipping is correctness-safe but is now the strongest measured
avoidable materialization cost. The replacement must derive the complete set
of attributes the executor can read, not merely restore the rejected
target-list-only shortcut. If the ordering-only proof below does not hold, the
standard lane's honest exact mask is `{id, embedding}` and the candidate can
save only the non-vector columns; the isolated A/B, not the SQL projection,
decides whether that smaller reduction is useful.

## Goal

Implement and measure a fail-closed, qual-aware payload attribute mask that
ships only columns proved necessary for executor-visible expressions. A
relation whole-row Var or an unproved expression shape selects all-column
shipping. A relation system-column Var keeps the existing plan-time
`EC_UNSUPPORTED_PROJECTION` error: remote `ctid`/`xmin` and the other system
columns cannot be reconstructed correctly by all-column shipping.

## Entry gate

1. Task 217's same-generation arm attestation remains mandatory.
2. The control is the conforming sharded owner path with lazy-10, schema cache,
   Algorithm-1 pushdown, and current gateway-copy behavior.
3. Pre-register the attribute-derivation contract and correctness matrix before
   measuring latency.

## Scope

### P1 — Attribute-use contract and observability

- Before deriving a mask, capture `EXPLAIN (VERBOSE)` and emitted tree/attnum
  evidence for the standard latency query. Record its actual executor target
  list, including the ordering expression injected above the CustomPath, rather
  than inferring requirements from the visible SQL projection. PG18 can invoke
  `plan_custom_path` with a NIL callback `tlist` under `CP_IGNORE_TLIST`, then
  replace the projection-capable CustomScan's `plan.targetlist` afterward; the
  callback `tlist` is therefore not the final derivation surface.
- Split derivation at the two points that hold the required facts. At plan time
  in `plan_custom_path`, mechanically prove or reject the ordering-only
  exemption from the original Query/pathkeys context and serialize only that
  proof (indexed heap attnum plus distance operator) into `custom_private`.
  At `begin_custom_scan`, derive the typed mask from the now-final three trees:
  `plan.targetlist`, `plan.qual`, and `custom_exprs[0]` (the ORDER BY query-value
  expression), applying the serialized exemption only to the one matching
  injected ordering expression. When the exact mask omits that vector, make an
  executor-local shallow `CustomScan` copy plus a deep target-list copy,
  replace the proven-unused expression in the private tree with a typed NULL,
  and rebuild the scan projection during `BeginCustomScan`. Never mutate the
  shared/cached plan tree. If copying or matching the one ordering entry fails,
  recompute the exact mask with the vector retained; merely leaving the
  expression to evaluate against an unshipped vector is unsafe. There is no
  separate recheck tree: EPQ and multi-window qual rejection both re-evaluate
  `plan.qual`. Assert that `custom_exprs[0]` is Const/Param-only and contains no
  relation Var; fail closed to all columns if that invariant ever changes.
- Export the result as a typed reusable API, not logic embedded in
  `build_payload_metadata`: `Exact(attnums)` versus
  `AllColumns(FallbackReason)`. Preserve the distinction even when an exact
  wide query names every live attribute, because Tasks 223 and 229 may consume
  only a proved-exact mask. Retain the sorted, deduplicated runtime value in
  executor state so payload metadata and future Task 223/229 selectors consume
  the same typed result rather than re-deriving it.
- Include every positive base-relation Var in non-ordering target expressions
  and `plan.qual`. A whole-row Var (`varattno == 0`) or any tree/Var whose use
  cannot be proved selects `AllColumns(reason)`. A relation system-column Var
  (`varattno < 0`) is not a fallback reason: preserve the existing plan-time
  `EC_UNSUPPORTED_PROJECTION` error at both path and plan validation.
- **Ordering-only rule:** exclude the vector Var referenced only by the single
  resjunk distance-sort entry when, and only when, all of the following are
  mechanically established at plan time: the entry is resjunk and matches the
  query's sole `sortClause`; its expression is the same distance `OpExpr` used
  to bind this ec_distann index; the Var occurs nowhere in non-resjunk target
  expressions or `plan.qual`; the query is the supported plain-base-relation,
  single-ORDER-BY/LIMIT shape with no grouping, distinct, window, set-operation,
  row-locking, append, or parallel consumer; and the CustomPath advertises the
  query's exact `sort_pathkeys`, so PostgreSQL needs no Sort or MergeAppend to
  read the junk value. If every condition is not mechanically proved, include
  the vector in `Exact`; do not guess or fall back to an id-only mask.
- Emit requested attribute numbers, payload-column count, payload bytes, and
  the exact/all-columns variant plus fallback reason in benchmark-only
  evidence.
- A benchmark-only control GUC may force `AllColumns`, but it must also disable
  ordering projection elision so the control executes the historical
  all-column plan rather than a hybrid plan.
- Classify a true zero-payload/index-only path, but do not implement it unless
  every output and visibility value can be reconstructed without a row fetch.
  The row-tier fetch is also the tombstone/visibility check, so id-only output
  alone does not make a zero-payload path safe.

### P2 — Correctness and isolated 100k A/B

Exercise id-only; multi-column; `SELECT *`; qual-only columns; nulls; toasted
values; mixed local/remote rows; correlated LATERAL queries; multi-window qual
rejection; rescan; EPQ/concurrent UPDATE; and remote failure. Pin the
ordering-only boundary with `SELECT id, embedding <-> $q` (distance is visible,
so the vector ships), an ORDER BY operand also referenced by a qual (the vector
ships), and a correlated rescan whose query Param changes (the sort-only vector
may remain excluded only under the same proof on every rescan). Preserve the
existing system-column error cases.

Then run one same-generation 100k A/B in which only the payload attribute mask
differs. The candidate must return byte-identical result ids in byte-identical
order under the deterministic benchmark contract. Advance only if the warm
end-to-end mean improves by at least **1.0 ms/scan or 5%**, with no material
tail regression; otherwise close STOP after recording whether the observed
mask was id-only or `{id, embedding}` and the actual byte reduction.

### P3 — Full decision matrix

Advance only a candidate that clears the pre-registered P2 gate to the standard
10k/50k/100k release
`ecaz bench suite` matrix. Report recall/result identity, mean and tails,
payload columns/bytes, owner endpoint/SQL time, storage, topology, and
NFR-021/NFR-022 conformance.

## Non-goals

- Repeating Task 220's packed SQL or Task 218's typed-locator candidate.
- Omitting qual/recheck attributes to win a benchmark.
- Changing traversal, head policy, payload window size, or stored format.
- Treating a candidate win as a shipped default without a separate
  productionization disposition.

## Acceptance

1. The attribute-use contract is test-pinned and fails closed to all-column
   shipping for ambiguous shapes, while system columns retain their hard error.
2. All semantic/failure cases match the control, including qual behavior that
   broke the historical target-list-only attempt and the ordering-only
   adversarial cases.
3. The 100k A/B has byte-identical ids/order and proves whether column/byte
   reduction clears the 1.0 ms/scan-or-5% warm-mean gate without a material
   tail regression.
4. A useful candidate receives 10k/50k/100k recall, latency, and storage
   evidence; otherwise the task closes STOP without a matrix.

## Outcome

Implementation commit `c9f79be4a` completes the typed exact/all-columns mask,
executor-private ordering-expression elision, fail-closed vector retention,
system-column hard error, benchmark observability/control, and the P2 semantic
matrix. CLI commit `f1351d2db` corrects variant fixture-reuse attestation so a
suite variant with benchmark seed metadata can safely reuse its own fixture.
Focused PG18 extension coverage and the focused CLI unit test pass; details are
in packet 002.

The isolated 100k gate in packet 003 advances: recall and ordered predictions
are identical, warm mean improves 17.1 -> 10.7 ms (-37.43%), and payload bytes
fall 123,076.8 -> 66.6 per scan. The required packet 004 matrix independently
confirms the candidate at every scale:

| Scale | Recall control / candidate | Warm mean control -> candidate | Payload bytes/scan control -> candidate |
| --- | --- | --- | --- |
| 10k | 0.9990 / 0.9990 | 14.7 -> 8.76 ms (-40.41%) | 121,624.72 -> 65.80 |
| 50k | 0.9545 / 0.9545 | 16.8 -> 10.8 ms (-35.71%) | 123,842.80 -> 67.00 |
| 100k | 0.9290 / 0.9290 | 17.4 -> 11.6 ms (-33.33%) | 123,103.44 -> 66.60 |

Each scale has byte-identical ordered predictions, arm-identical storage, three
owners with no non-owned/orphan/coordinator-resident payload, and admissible
NFR-021/NFR-022 provenance. No implementation or measurement work remains;
the task awaits an outside verdict on review-open packets 002-004 and is not
self-marked review-closed.

Productionization disposition: Task 222 ships the proved exact payload mask as
the production default. That decision rests on byte-identical ordered results
and identical recall at 10k, 50k, and 100k, rather than a recall-for-latency
trade, so Task 219's recall-equivalence product-ruling clause is not engaged.
If a query shape escapes the ordering-only proof, execution fails closed to
all-column shipping, preserving the pre-Task-222 behavior.

## Required review packets

1. `reviews/task-222/001-plan/`
2. `reviews/task-222/002-contract-and-correctness/`
3. `reviews/task-222/003-isolated-100k/`
4. `reviews/task-222/004-full-scale-decision/` (only after a useful screen)

## References

- `reviews/task-218/001-production-profile-attribution/`
- `reviews/task-220/002-isolated-candidate/`
- `src/am/ec_distann/custom_scan.rs` (`build_payload_metadata`)
- Roadmap MAT-28 / MAT-29
