# Review request: Task 143 packet 002 — default flips landed + confirming cell

- Code commit: `815518d82` (branch `task-141-sdot-kernel`)
- Approval basis: packet 001 recommendations, operator-approved 2026-07-03.

## What changed

- `ec_ivf.turboquant_scorer` GUC default: `lut` → `int8_approx`.
- `dense_posting_blocks` reloption: default `-1` (auto) — resolves dense
  for the TurboQuant lane, row for RaBitQ (kept with the Task 111a
  scope); explicit 0/1 still override. The no-reloptions DEFAULT struct
  flips to dense. Unit tests keep pinning LUT via the cfg(test) accessor
  for legacy-kernel coverage (documented in-code).

## Confirming default-path cell (fresh 100k, NO reloptions, NO GUCs)

`artifacts/`: precheck shows `current_setting('ec_ivf.turboquant_scorer')`
= `int8_approx` at `815518d82`; the no-reloption build produced the dense
layout (index 81.7 MiB / 856.5 B/row — exactly the packet-001 dense size,
vs 90.4 MiB row).

| metric | packet-001 explicit dense-int8 | default path (this cell) |
|---|---|---|
| recall@10 nprobe 8..64 | 0.7844 / 0.8344 / 0.8750 / 0.8938 / 0.9031 / 0.9125 / 0.9250 | identical, point-for-point |
| latency mean n32 / n40 | 1.75 / 2.03 ms | 1.71 / 1.94 ms |
| storage | 81.7 MiB | 81.7 MiB |

The out-of-the-box path now IS the measured promotion candidate.

## Validation

cargo check clean; 199 am::ec_ivf tests pass; clippy pg18 carries only
the pre-existing finding (`artifacts/` logs in packet 001 lineage; the
flip commit message records the run).
