# Review Request: C1 AVX2 No-QJL 4-bit Score

## Context

Packet `265` removed unused query-prep state from the tiled `1536x4-bit`
no-QJL path and improved the verified warm steady-state `10K`, `m=8`,
`ef_search=40` cell from about `14.19ms mean` to `11.11ms mean`.

That is real progress, but it is still well above the warm-side `NFR-001`
target. Reviewer feedback on packets `257` and `264` both pointed at the same
remaining hotspot: the no-QJL `4-bit` scorer is still scalar on x86 even
though this production lane dominates current warm measurements.

## Problem

In `src/quant/prod.rs`:

- `score_ip_from_split_parts_no_qjl_4bit(...)` walks packed bytes one by one
  and scores two nibbles at a time with scalar loads and scalar multiplies
- the hot `1536x4-bit` production lane reaches that path whenever QJL is
  disabled
- the existing AVX2 scoring machinery only accelerates the QJL-enabled path

So the current warm path still leaves SIMD throughput on the table exactly
where the active production lane spends its scoring time.

## Planned work

1. Add an AVX2 fast path for no-QJL `4-bit` scoring on x86_64.
2. Keep scalar fallback behavior identical for non-AVX2 or non-x86 targets.
3. Extend test coverage so the AVX2 path remains numerically aligned with the
   scalar reference.
4. Re-run the full checkpoint gate and the same verified warm per-cell `10K`
   cell to measure macro impact.

## Exit criteria

- no-QJL `4-bit` scoring dispatches to AVX2 when available
- scalar and AVX2 paths agree on representative production-dimension cases
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- verified warm rerun recorded against the same `10K`, `m=8`, `ef_search=40`,
  `warm-after-prime3`, `per-cell` seam
