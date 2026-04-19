# Review Request: C1 Task16 Tqvector Compact Artifact Layout

Current head at execution: `5ff3703`

## Context

Packets `442` and `443` fixed the product model:

- `ecvector` is the canonical row type
- `tqvector` is the TurboQuant-family sibling artifact

But one important mismatch remained:

- `tqvector` was only a sibling in taxonomy
- it still carried the older per-row wire layout:
  - `dim`
  - `bits`
  - `seed`
  - `gamma`
  - code bytes

That was the wrong storage contract for an explicit family-specific artifact.
The artifact should stay self-describing enough for SQL/operator use, but it
should not burn dead bytes in every row.

Reviewer feedback on `442`/`443` also called out one missing regression:

- when a table carries both indexed `ecvector` and sibling `tqvector`,
  default `pq_fastscan` heap-f32 behavior must stay pinned to the indexed
  `ecvector` column

This slice lands both pieces together.

It also fixes the repo front door:

- `README.md` now documents `ecvector` as the canonical row type
- quick-start examples build `tqhnsw` indexes on `ecvector`, not on
  `tqvector`
- the README explicitly calls `tqvector` a TurboQuant artifact/debugging
  surface rather than a normal row column

## What changed

### 1. `tqvector` is now a compact canonical TurboQuant artifact

`src/lib.rs` changes the persisted `tqvector` contract from:

- `dim + bits + seed + gamma + code bytes`

to:

- `dim + gamma + code bytes`

with:

- `bits = 4` enforced at the type surface
- `seed = 42` enforced at the type surface

Concretely:

- `pack` / `unpack` no longer persist `bits` or `seed`
- `parse_text` still accepts the existing text surface but rejects
  non-canonical `bits` / `seed`
- `encode_to_tqvector(...)` now rejects non-canonical artifact requests
  before row insertion
- `tqvector` SQL operators still work because `dim` remains inline and
  `bits` / `seed` are known invariants

This shrinks the raw datum overhead from:

- `15` bytes per row (`dim + bits + seed + gamma`)

to:

- `6` bytes per row (`dim + gamma`)

before the packed code bytes.

### 2. Canonical/sibling separation is now locked in by pg regression

Added:

- `test_pq_fastscan_indexed_ecvector_ignores_tqvector_sibling`

The test creates a mixed table:

- `artifact tqvector`
- `embedding ecvector`

Then it:

1. builds a `pq_fastscan` index on `embedding ecvector`
2. asserts runtime settings still report:
   - `pq_fastscan_rerank_mode = heap_f32`
   - `pq_fastscan_rerank_mode_resolution = default_heap_f32_with_indexed_column`
   - `pq_fastscan_rerank_source_column IS NULL`
3. asserts emitted exact scores match the indexed `ecvector` values, not the
   sibling `tqvector` artifact payload

### 3. Encoder round-trip is explicit now

Added:

- `test_encode_to_tqvector_round_trips_canonical_artifact_layout`

This proves `encode_to_tqvector(...)` emits the compact canonical artifact bytes
that the SQL type surface persists.

## Important design note

The original checklist sketch talked about hoisting all of
`(dim, bits, seed)` out of the row and leaving only `gamma + code bytes`.

That was not viable on current head without a much larger SQL/type refactor,
because PostgreSQL output/operator functions for `tqvector` do not receive
typmod. A pure typmod-only artifact would have broken the current sibling SQL
surface.

Current head therefore makes the compact compromise:

- keep `dim` inline
- make `bits` and `seed` canonical invariants
- drop the dead per-row `bits` / `seed` bytes

## Why this matters

This slice keeps the row model coherent for future work:

- `ecvector` remains the one canonical exact row type
- `tqvector` remains a narrow explicit artifact/debugging type
- the artifact is now actually compact, not just renamed

That is the right base for:

- task-16 measurement closure on the corrected surface
- native HNSW build work
- eventual `hnsw_rs` removal
- future pg18 integration without reopening the row-model split

## Validation

Ran on this exact tree:

```bash
cargo test test_pq_fastscan_indexed_ecvector_ignores_tqvector_sibling -- --nocapture
cargo test test_tqhnsw_insert_rejects_mismatched_seed -- --nocapture
cargo test test_binary_recv_rejects_truncated_bytes -- --nocapture
cargo test test_non_empty_index_build_spans_multiple_data_pages -- --nocapture
cargo test test_encode_to_tqvector_round_trips_canonical_artifact_layout -- --nocapture
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

## Review focus

1. Is the compact `tqvector` contract the right compromise for current head:
   inline `dim`, canonical `bits=4` / `seed=42`, and no per-row `bits` / `seed`
   bytes?
2. Do the new regressions adequately lock in:
   - canonical `ecvector` vs sibling `tqvector` separation
   - canonical artifact encoder round-trip behavior?
3. Is any remaining doc/runtime text still overstating a future typmod-only
   `tqvector` layout that current head does not actually ship?
