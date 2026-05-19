# Artifact Manifest: IVF Unsafe Burndown Closeout

Head SHA: `1d2285b6` (state at packet authoring).

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/122-ivf-closeout/`

Packet type: retroactive closeout / coverage summary. No code or
baseline files modified.

Surface: PG18, IVF access method (`src/am/ec_ivf/*`). No corpus,
no benchmark.

## Artifacts

- `ivf-coverage-table.md` — production file coverage table and
  per-wave subtotals across the 19 IVF burndown packets (024,
  025–035, 036–039, 040–042).
- `ivf-invariant-summary.md` — IVF safety invariant graph, lock/
  WAL summary, RAII guard inventory, posting-list and centroid
  specifics, and Task 50 candidate list.
- `unsafe-audit.log` — `bash scripts/check_unsafe_comments.sh`,
  passed.
- `unsafe-baseline-report.log` — `bash scripts/unsafe_baseline_report.sh`,
  `entries: 0`, `files: 0`.
- `ivf-source-remaining-baseline.log` — `src/am/ec_ivf` residual
  entry count (`0`).
- `codex-review-unsafe-audit.log` — independent reviewer rerun of
  `bash scripts/check_unsafe_comments.sh`, passed.
- `codex-review-baseline-report.log` — independent reviewer rerun
  of `bash scripts/unsafe_baseline_report.sh`, `entries: 0`,
  `files: 0`.
- `codex-review-packet-count-check.log` — independent reviewer
  check that the corrected packet docs no longer claim `21`
  packets.
- `codex-review-diff-check.log` — `git diff --check`, passed.

## Key result lines cited by `request.md`

- Global baseline: 0 entries / 0 files.
- IVF production source residual: 0 entries.
- IVF Task 35 production-source clearing: 326 entries across 19
  packets (327 under absorbed-drift accounting, with packet 022's
  +1 page.rs line-drift artifact reconciled).
