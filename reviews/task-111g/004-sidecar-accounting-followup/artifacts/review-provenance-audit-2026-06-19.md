# Review Provenance Audit — 2026-06-19

Reviewer: Codex / GPT-5

Scope: branch `bench-ivf-111g-115-attribution` at `ed3302bbb`, including local
untracked benchmark artifacts.

## Observations

- `git ls-files benchmarks/ivf-111g-115-attribution` shows the committed
  benchmark packet currently includes:
  - `FINDINGS.md`
  - `manifest.md`
  - `configs/*.json`
  - only the `artifacts/head-constant/*` result/log set
- `git status --short --untracked-files=all` shows the result/log sets cited by
  the latest task verdicts and reviewer feedback are still untracked, including:
  - `artifacts/head-rerank-format-matrix/*`
  - `artifacts/head-sidecar-index-placement/*`
  - `artifacts/head-lazy-ab/*`
  - `artifacts/head-prune-ab/*`
  - `artifacts/head-dense-prune-ab/*`
  - `artifacts/head-residual-ab/*`
  - `artifacts/head-quant-bits-matrix/*`
  - `artifacts/head-constant-rabitq/*`
  - `artifacts/hist-baseline/*`
- `benchmarks/ivf-111g-115-attribution/manifest.md` still lists the common
  nprobe sweep as `[8,16,24,32,48,64]`, while `FINDINGS.md` and the current
  configs use `[8,16,32,64,128,200]`.
- The same manifest's verdict table still contains `_tbd_` rows even though
  `FINDINGS.md` and the task docs now make final benchmark verdict claims.
- `benchmarks/ivf-111g-115-attribution/run-historical.sh` is untracked and is
  named by `FINDINGS.md` as the historical attribution driver.

No new tests or benchmarks were run for this review.
