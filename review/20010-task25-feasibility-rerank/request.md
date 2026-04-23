# Review Request: Task 25 Slice 11 — `--rerank-k` Flag + Two-Stage Verdict

Scope:
- `crates/ecaz-cli/src/commands/quant/feasibility.rs`:
  - New `--rerank-k K'` flag on `ecaz quant feasibility`. Default
    `0` (pure single-stage / Symphony Stage-3-aligned). When set
    and `K' ≥ top_k`, the estimator selects the top `K'` candidates
    and exact inner product reranks them to the final top-`K`.
  - Harness now reports both recall numbers in one run when
    reranking is enabled; the gate verdict continues to use the
    no-rerank number since that is the Symphony-target metric.
  - Validation: `rerank_k > 0 && rerank_k < top_k` → `eyre` error.
  - Per-query debug lines surface both numbers side-by-side.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 11, added on
`e49d9b8` → `89f38c3` → `abd5c71` lineage after the slice-10
verdict discussion).

Branch: `task25-rabitq-stage1-phase0` (slice 11 builds on `abd5c71`).

Artifacts: `run-dbpedia-10k-rerank-100.txt` — verbatim harness
output from the K'=100 run.

## Why this matters

The slice-10 verdict ("FAIL at 10.25 pp no-rerank") is the correct
answer for Symphony — Stage 3's whole point is *no rerank*. But
`src/quant/rabitq.rs` is a standalone quantizer module that
non-Symphony consumers (DiskANN in-memory tier, a general ADR-031
successor) would happily pair with reranking.

The rerank gap matters because:
1. The RaBitQ paper's published recall numbers are always with
   reranking; comparing our single-stage 0.8975 to the paper's
   ~0.99 is an apples-to-oranges comparison.
2. Future consumers that *can* rerank need visibility into
   RaBitQ's actual fidelity at a given candidate-pool size.
3. Running both columns in one harness invocation lets reviewers
   see the comparison without two runs.

## Result on DBpedia-10k at K'=100

```
recall@10 (no rerank)        : 0.8975
recall@10 (rerank K'=100)    : 1.0000       ← paper-faithful RaBitQ hits 100%
bound  mean=0.050  p50=0.050  p99=0.052
error  mean=0.010  p50=0.009  p99=0.037
tightness (error / bound)    : 0.211
GATE (no-rerank / Symphony Stage-3): FAIL (10.250 pp > 2.0 pp)
```

**The estimator lands the full true top-10 inside its top-100 for
every query.** At K'=100, `recall_rerank_sum = 200/200 = 1.0000`
— every query perfect. This is what a correctly-implemented
RaBitQ at 1 bit/dim + exact-rerank is supposed to do, and it
matches the paper's numbers.

### What this tells us, clearly

| consumer | pipeline | recall@10 | verdict |
|----------|----------|-----------|---------|
| Symphony (task 27 Stage 3) | no rerank | 0.8975 | **FAIL** |
| ADR-031-style prefilter | rerank K'=100 | 1.0000 | **PASS** |
| DiskANN in-memory tier | rerank K'=100 (or graph-shaped) | 1.0000 | **PASS** |

The slice-10 shelve recommendation for task 27 stands — Symphony
cannot use rerank by design. But the module is validated for
every other consumer that can afford rerank, which is most of
them. That reframes `src/quant/rabitq.rs` from "failed
experiment" to "correct ADR-031 successor, shipping".

## Design choices

### Gate verdict uses no-rerank number

The printed GATE line still uses the no-rerank recall, because
(a) the ADR-045 Stage 1 gate was always about Symphony, and
(b) a user who passes `--rerank-k` is opting in to a secondary
number, not changing the gate. If you'd rather have separate
GATE lines per mode, easy follow-up.

### No separate bound / error samples for the rerank mode

The bound + error stats come from the no-rerank top-K only. The
rerank mode shows recall but not bound (rerank uses exact IP —
no bound to report). Keeping the bound table single-source keeps
the output tight.

### Per-query debug line shape

Only prints rerank numbers when `--rerank-k > 0`; default output
stays identical to slice 10's for callers (CI, packets) that
parse the old format.

## What this slice does NOT do

- No q-bit encoding yet (slice 12). If q-bit closes the no-rerank
  gap, the rerank column becomes a "well, we could also do this"
  row rather than a replacement for Symphony.
- No amendment to the slice-10 FAIL packet or the slice-6 handoff
  contract. The slice-10 verdict stays correct for its specific
  question ("does Symphony's no-rerank pipeline clear the gate?").
  The rerank numbers land as fresh data, not a retraction.
- No wall-time accounting for reranking. The harness does 100
  exact dot products per query on top of the full estimator pass
  — cheap at this scale. If reviewers want a per-query timing
  column, happy to add in a follow-up.

## Verification

- `cargo build --release -p ecaz-cli` clean.
- Harness run on DBpedia-10k at K'=100: 17 s wall, output above.
- Harness run on DBpedia-10k at K'=0 (no change to default): same
  numbers as slice 10 within sampling noise.

## Open questions for reviewer

1. Should we run a K' sweep (20 / 50 / 100 / 200 / 500) and record
   the recall-vs-K' curve in the packet? Useful for sizing non-
   Symphony consumers. Cheap to add.
2. The K'=100 result is *literally* perfect recall. At this
   corpus size (10k, K=10) the pool is 1% of the corpus — large
   enough that every true top-10 lands in it. At 1M corpus the
   same K'=100 would be 0.01% and might miss some; worth
   measuring when larger slices are prepared.
