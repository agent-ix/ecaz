# Task 98 Packet 001: Phase A Design + Instrumentation Slices

Design packet (no new kernel code yet) plus two already-landed
instrumentation slices it depends on:

- `ed0fe1a23` — `ec_hnsw.turboquant_exact_score_mode` GUC (the TiledLut /
  Int8Approx bench surface; resolves the Task 87 zero-counter context
  called out in the task's references before any kernel work).
- `fb7083c78` — per-flush batch-width histogram on the block-kernel
  counters + SQL/CLI surfacing (acceptance criterion 4's data source),
  recorded by all four existing shared wrappers.

`artifacts/design.md` specifies the two kernel families, their parity
contracts (integer-exact for int8_approx32; forced-scalar anchor +
envelope for tiled_lut32), the HNSW routing/gating, and how the width
histogram feeds the Phase A scope-down decision.

## Validation

Both commits validated in their messages (clippy clean; candidate_batch
10/10 with width assertions, ec_hnsw 251/251 for the GUC slice, CLI bench
module 157/157).

## Review request

Please review the design and the two instrumentation slices. Next packet:
scalar kernels + HNSW routing + the real10k/50k/100k width-distribution
cells.
