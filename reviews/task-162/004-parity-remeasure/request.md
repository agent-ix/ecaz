# Review request — Task 162 packet 004: feedback fixes + parity re-measure

- Branch: `task-162-ec-distann-m0`, head `fb1f67eb6`
- Responds to: `reviews/task-162/003-g0-killcheck/feedback/2026-07-07-01-reviewer.md`
  (all four findings; commit map in
  `003-g0-killcheck/feedback/2026-07-07-02-coder.md`)
- Evidence: `artifacts/manifest.md` + `artifacts/results.jsonl` +
  current-head clippy/pg_test logs.

## What this packet shows

1. **Finding 1 closed**: current-head validation is green and packet-local
   (clippy clean; `cargo pgrx test pg18 ec_distann` 45/45 at `fb1f67eb6`).
2. **Finding 2 closed**: rabitq is the code default, pinned by test and
   recorded in ADR-085 D7.
3. **Finding 4 closed**: LIMIT > top_k is correct via iterative deepening
   (regression-tested); `top_k` demoted to a performance hint.
4. **Finding 3 measured, not closed**: the endorsed levers (batched
   neighbor scoring, heap prefetch) landed and are **inert on the warm
   50k tail** — the 0.995 point re-measures at **2.03×** (13.6 vs
   6.72 ms). The manifest's arithmetic attributes the tail to the D11
   `records read == reranked` contract (an exact heap read per expansion,
   ~200 at top_k=200, vs diskann's 64-candidate rerank budget) plus
   one-record-per-page reads at R=32/dim 1536. Parity through the ~0.988
   band holds at ≤1.16× on both scales; 10k remains dominant below
   0.9995.

## Decision needed (operator/reviewer): disposition of the 50k ≥0.995 gap

Manifest lists the three options with costs: (1) banded M0 exit with the
gap documented as a D11 cost and deferred to the M4 anchor-based gate
(distann's absolute 0.9950@13.6ms already beats the 37.6 ms IVF anchor);
(2) an R=16 record-packing A/B (halves record I/O, graph-quality risk);
(3) a spec-level rerank-economy amendment to FR-079/D11 (diskann-style
exact-read budget) — architecture change, not a tweak.

My recommendation is (1): the M0 milestone's purpose (de-risk format,
head index, loop shape with the sibling as control) is met, the G0
kill-check is a GO on the numbers that matter to the program gate, and
options (2)/(3) are better decided with M1 stitch data in hand. I have
not implemented (2) or (3) — both change measured surfaces and need an
explicit go-ahead.
