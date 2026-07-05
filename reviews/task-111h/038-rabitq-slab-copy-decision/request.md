# Review Request: Task 111h RaBitQ Slab-Copy Decision

This packet requests review for a read-only Task 111h closeout decision over the
remaining compact index copy/slab checklist row.

Packet:

- `reviews/task-111h/038-rabitq-slab-copy-decision/`

What it concludes:

- f16 is already no-slab in the compact index path because it scores scalar
  borrowed payload slices.
- TurboQuant is already no-slab in the compact index path because packet `032`
  added borrowed payload batch scoring and validated
  `rerank_payload_slab_bytes_copied == 0`.
- RaBitQ4/8 should keep the current contiguous survivor scoring slab for Task
  111h because that slab feeds the measured fast arithmetic estimator. The
  available in-tree borrowed candidate-batch route uses the multi-bit block
  kernel, which Task 106 measured slower for bits=4 on both M5 and Intel.

Evidence:

- `artifacts/rabitq-slab-copy-decision.md`
- `artifacts/manifest.md`
- cited packets:
  - `reviews/task-106/001-m5-multibit-rabitq-bench/`
  - `reviews/task-106/002-intel-avx2-bench/`
  - `reviews/task-111h/030-counter-fixture-closeout-audit/`
  - `reviews/task-111h/032-turboquant-borrowed-rerank/`
  - `reviews/task-111h/036-rabitq8-score-clip-ab/`

Validation:

- No new runtime tests or benchmarks were run. This is an evidence packet over
  existing source, PG18 counter fixtures, and packet-local Task 106/111h
  benchmarks.

Review focus:

- Confirm it is acceptable to close the Task 111h slab-copy row with this
  format-specific split: implemented for f16/TurboQuant, explicitly benchmarked
  away for RaBitQ4/8.
- Confirm the packet does not overclaim: a future borrowed arithmetic-estimator
  API may still be worthwhile, but the current borrowed block-kernel route is
  not the right replacement for the 111h RaBitQ4/8 decision path.
