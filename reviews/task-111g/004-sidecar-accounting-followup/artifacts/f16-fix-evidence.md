# f16 rerank subnormal-decode fix — evidence

Packet-local record of the f16 fix (codex 111g/004 P1#4: exact commit, command,
test output).

- **Commit:** `0c3efc611` on `bench-ivf-111g-115-attribution`
  ("fix(ec_ivf): correct subnormal binary16 decode in f16 rerank").
- **File:** `src/am/ec_ivf/rerank.rs` — `f16_bits_to_f32` subnormal branch: base
  unbiased exponent `-14` (was `-1`), drop the spurious `+1` in the f32 rebias.
- **Bug:** subnormal binary16 (|x| < 2⁻¹⁴) decoded ~2¹⁴× too large
  (`3.9e-5 → 0.64`); near-zero embedding components became ~0.64 garbage →
  f16-rerank recall@10 collapsed to ~0.6.

## Tests (unit — do not install a .so, safe vs bench host)

```
cargo test --no-default-features --features pg18 --lib rerank::tests
```

Result: **13 passed; 0 failed** — incl. the three new regressions added with the
fix:
- `f16_round_trip_is_accurate_for_subnormal_magnitudes` (vs numpy float16 refs)
- `f16_bits_match_numpy_reference_for_embedding_magnitudes`
- `f16_scorer_ranking_matches_f32_on_realistic_vectors` (top-10 overlap ≥9/10;
  reproduced the collapse at 0/10 before the fix)

## SQL-level proof (end-to-end, on the fixed release `.so`)

`benchmarks/ivf-111g-115-attribution/artifacts/head-rerank-format-matrix/results.jsonl`
— f16 table-side recall@10 @100k: 0.964 (np64) → 0.9975 (np200), matching f32
(0.965 → 0.9985). Before the fix: ~0.61 flat (see FINDINGS Finding 1).

A full packet-local pgrx test log will be captured (`cargo pgrx test pg18` for the
rerank set) once the historical bench frees the machine; the unit + SQL evidence
above already establishes the fix.
