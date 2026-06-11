# Task 104 packet 008 — closeout artifacts

- Task bucket: `reviews/task-104/`; packet `008-m5-matrix-closeout/`
- Branch: `task-104-m5-bench-optimization`; date 2026-06-11
- Deliverable: `../m5-index-quant-option-matrix.md` (Task 99
  Apple-silicon supported-target column).
- `recall-on-off-parity.txt` — recall@k equality across every measured
  kernel/batch on-vs-off cell pair, computed from the packet 002-007
  `results*.jsonl` files (40 pairs, 0 mismatches).
- `floor-gate-summary.txt` — aggregated kernel vs anchor ns/candidate per
  (suite, quant) from the same sources. Caveat: the "anchor" column is the
  off-path one-off accounting (labeled isa=scalar by the counter surface;
  internally NEON-accelerated for TQ/QJL one-off scoring), making the
  floor ratios conservative. Families without one-off rows
  (rabitq/int8/hamming/grouped-PQ) anchor on the Task 93/95/98 M5
  closeouts plus this matrix's e2e on/off deltas.
- Sources of truth per cell: packets 002-007 suite manifests, results
  files, and per-cell logs (see each packet's `artifacts/manifest.md`).
