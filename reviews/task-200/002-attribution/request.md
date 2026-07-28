---
agent: codex
role: coder
model: GPT-5
date: 2026-07-28
seq: 2
---

# Task 200 attribution — root cause found

The held-transaction production A1 remains bounded: 300 ordinary physical
latency queries on one backend stayed at 252,360–259,596 KB RSS. The leak is in
the benchmark-only owner seed path, not `PhysicalGenerationScan::open`:

- one and 200 isolated `PhysicalGenerationScan::open` calls both ended at
  `TopTransactionContext: 142606336 total`;
- 20 repeated `owner_scan_seed_candidates` calls reached
  `TopTransactionContext: 5595201536 total`;
- the exact leaking operation was pgrx `value::<Vec<u8>>()` on `graph_record`.
  Its bytea conversion detoasts into the transaction context and retains each
  copy until commit.

The fix is implemented in the next packet: read the raw SPI datum and wrap it
in the repository `DetoastedVarlena` guard, which frees each detoast copy after
decoding. The attribution endpoints are benchmark-feature-only.

See [`artifacts/physical-open-1.log`](artifacts/physical-open-1.log),
[`artifacts/physical-open-200.log`](artifacts/physical-open-200.log),
[`artifacts/owner-seed-20.log`](artifacts/owner-seed-20.log), and
[`artifacts/attribution-node1-postgres.log`](artifacts/attribution-node1-postgres.log).
