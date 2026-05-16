# Task 47: Recall and Cost-Model Correctness Gates

Status: **proposed** — turns recall and planner-cost behavior into
machine-verifiable gates, so the things that *matter to users* (search
quality and plan stability) are protected from regression even when
correctness-adjacent code changes.

## Scope

Three correctness gates that ECAZ does not enforce in CI today:

1. **Brute-force exact KNN differential.** For a fixed corpus, compute
   the exact top-k by linear scan; assert each ECAZ AM's recall ≥ a
   per-AM floor with documented confidence intervals.
2. **Cross-AM consistency.** For the same corpus and query, assert the
   AMs agree on top-k membership up to documented quantization /
   pruning differences. Disagreements above a threshold are filed for
   review.
3. **Cost-model regression gate.** For a fixed query set, capture the
   planner's costed plan tree per query; assert the cost values do not
   drift beyond a per-query band without an explicit packet.

## Why

End-to-end recall and cost are the user-visible contracts. They depend
on dozens of internal subsystems (SIMD scoring, codebook construction,
graph build, scan accumulator, candidate merge, cost model), any one of
which can silently degrade without breaking a unit test:

- A SIMD bug that drops 0.5% recall passes everything in Task 36 if the
  tolerance is loose; the recall gate catches it.
- A planner cost tweak that flips ECAZ to seq-scan on small tables is
  semantically correct but operationally bad; the cost gate catches it.
- A change in tie-breaking inside a candidate priority queue can swap
  result order without changing membership; the cross-AM consistency
  check distinguishes "different scoring path" from "different result
  set."

Today recall is measured in `recall_integration` tests behind
`--ignored`, run manually. Cost-model behavior is not gated at all —
only `spire_cost_tuning.rs` exercises adjacent paths. End-to-end
correctness is the layer where bugs that escape every other lane are
caught; it should be a first-class lane, not a manual one.

## Approach

1. **Fixture corpora.** Use the existing fixture set under `fixtures/`
   (m5_diskann_real10k, real100k, synth10k) and add small "gate
   corpora" sized so the gates run in PR-CI budget:
   - 1k–5k vectors per fixture,
   - mixed-distribution synthetic (uniform, clustered, adversarial),
   - real-data subsets from existing fixtures (subsample with fixed
     seed).
2. **Exact KNN baseline.** A small Rust helper computes brute-force
   top-k for each `(query, k)` pair and caches the result under
   `fixtures/exact-knn/`. Cache keyed on `(corpus_hash, query_hash, k,
   metric)`; regeneration only on hash change.
3. **Per-AM recall gates.** For each AM at each `(k, search_breadth)`:
   - measure recall = |ECAZ_topk ∩ exact_topk| / k,
   - assert ≥ floor with a confidence band derived from query count,
   - report observed recall in the packet.
   Floors documented per AM in `docs/recall-floors.md` and updated only
   by review-packet decision.
4. **Cross-AM consistency.** Compute Jaccard / Kendall-tau between AM
   top-k results on shared corpus; assert against a per-pair threshold.
   Disagreements above threshold do not fail CI by default but are
   logged and surfaced in the packet — many are expected (different
   quantizers, different pruning).
5. **Cost-model gate.** A fixed query suite saved as
   `fixtures/cost-queries/*.sql`. CI captures `EXPLAIN (FORMAT JSON,
   COSTS ON)` output, normalizes path identifiers, and diffs against
   committed `fixtures/cost-queries/baseline.json`. Drift > X% per
   node-cost requires a packet update of the baseline.
6. **Operator override.** A `--accept-drift` flag for the gate runner
   accepts the new baseline and writes it to the fixture; updates land
   in a packet with rationale.
7. **Make lanes:**
   - `make recall-gate` — PR-CI: small corpus, per-AM floor check.
   - `make recall-gate-full` — nightly: larger corpora, full sweep.
   - `make cross-am-gate` — PR-CI: Jaccard / Kendall-tau report.
   - `make cost-gate` — PR-CI: EXPLAIN diff vs. baseline.

## Validation

- `make recall-gate` runs in under 5 minutes against the small gate
  corpora.
- A deliberately introduced SIMD bug (paired with Task 36) is caught
  here as an end-to-end recall drop; a deliberately introduced cost
  bias (e.g., divide cost by 10) is caught by `cost-gate`.
- Cross-AM consistency report produces stable Jaccard values for a
  no-op PR.
- Baseline regeneration works and produces a reviewable diff.

## Exit Criteria

- `make recall-gate` runs in PR-CI with documented per-AM floors.
- `make cost-gate` runs in PR-CI with a committed baseline that updates
  via explicit packet.
- `docs/recall-floors.md` and `fixtures/cost-queries/` are authoritative.
- The existing `recall_integration` tests are either retired or marked
  redundant in favor of these gates.

## Dependencies

- Pairs with Task 36 (SIMD diff) — that catches the *unit* divergence,
  this catches the *end-to-end* impact.
- Independent of Tasks 37–46 mechanically; useful in parallel.
- Needs the same live PG18 environment as Tasks 37–38 for the cost gate.
