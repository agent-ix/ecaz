# Task 105 packet 006 — aggregation artifacts manifest

- Task bucket: `reviews/task-105/`, packet `006-full-scale-matrix`
- Kind: aggregation/analysis packet — **no new measurements**; every
  number is derived from committed packet-local artifacts or quoted
  from pinned baseline packets.
- Code analyzed: `main=1345ca603` (the head both lanes measured);
  packet authored on branch `task-105-full-sweep`.

## Artifacts

- `full-scale-matrix.md` — the Phase 3 matrix document
  (scale × AM × quant × option × lane), findings, honest markers,
  AC mapping.
- `baseline-comparison.md` — IVF rabitq1/TQ vs May 2026 baselines and
  the pinned comparator mapping (no re-runs).
- `handoff-release-readiness.md` — handoff note for the
  safety/cleanup/release track.
- `gen_matrix.py` — deterministic table generator; run from repo root.
- `matrix-tables.md` — verbatim generator output (the tables embedded
  in `full-scale-matrix.md` between the BEGIN/END generated markers,
  plus the kernel-share table computed by the same method described
  below).

## Data sources (all in-repo)

| source | path |
|---|---|
| G4 10k | `reviews/task-105/004-g4-lane/artifacts/sweep-10k/20260612T054423Z/results.jsonl` |
| G4 50k (warm, citable) | `reviews/task-105/004-g4-lane/artifacts/sweep-50k-warm/20260612T074213Z/results.jsonl` |
| G4 100k confirm gate | `reviews/task-105/004-g4-lane/artifacts/gate-clean/20260612T053900Z/results.jsonl` |
| G4 1M | `reviews/task-105/004-g4-lane/artifacts/sweep-1m/20260612T133752Z/results.jsonl` |
| Intel 10k | `reviews/task-105/005-intel-lane/artifacts/sweep-10k-clean/20260612T061932Z/results.jsonl` |
| Intel 50k | `reviews/task-105/005-intel-lane/artifacts/sweep-50k-quiet/results.jsonl` |
| Intel 1M | `reviews/task-105/005-intel-lane/artifacts/sweep-1m/results.jsonl` |
| 100k full on/off profile | `reviews/task-99/008-g4-lane/` (neoncap-run), `reviews/task-99/009-intel-lane/` (profile-run) |
| May scaling baseline | `benchmarks/cloud-scaling-multi-am/manifest.md` (SHA `775455dc`, 2026-05-17) |
| May 1M final gate | `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md` (head `902e8e066`) |
| Pinned comparators | `benchmarks/comparators-50k-100k-1m/manifest.md` (head `63024cce`) |

## Methods

- Latency/recall/ISA/storage tables: `gen_matrix.py` (step-name parse,
  matched sweep points, on/off pairing; recall mismatches flagged
  inline — none present).
- Kernel scoring share: per kernel-on step,
  `sum(block_kernel_counters.kernel_elapsed_ms)` over
  `latency mean × count` at the same sweep label.
- Auto-nlists geometry for the baseline comparison: confirmed from
  `src/am/common/training.rs::resolve_auto_nlists`
  (ceil(sqrt(rows)), so ≈995 at 990k rows) and the May gate precheck
  log showing no `nlists` reloption on the preserved index; t105
  fixtures pin `nlists=256`
  (`reviews/task-105/002-full-scale-sweep-configs/artifacts/t105-fixtures-1m.sql`).
