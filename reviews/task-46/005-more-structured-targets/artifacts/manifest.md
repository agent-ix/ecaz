# Packet 005 — Task 46: more structured fuzz targets

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/005-more-structured-targets/`
- Validation head SHA: code commit landing the three new bins +
  Cargo.toml registration.
- Branch: `main`
- Surface under validation: three new libFuzzer targets, plus the
  closure of Task 46 §Exit Criteria #1.

## What changed

| Path | Kind | Purpose |
|---|---|---|
| `fuzz/Cargo.toml` | manifest | three new `[[bin]]` entries |
| `fuzz/fuzz_targets/topk_merge_structured.rs` | new target | §Approach 2.a property test |
| `fuzz/fuzz_targets/quant_encode_decode_roundtrip.rs` | new target | §Approach 2.c roundtrip |
| `fuzz/fuzz_targets/vector_normalize_structured.rs` | new target | structured sibling of `fuzz_vector_normalize` |

## Artifacts

### fuzz-topk-merge-10s.log

- Command: `PATH=…nightly… cargo fuzz run fuzz_topk_merge_structured
  -- -max_total_time=10 -print_final_stats=1`
- Timestamp: 2026-05-26
- Result: 850,323 runs / 10s (77k exec/s), 150 new units, 0 crashes.

### fuzz-quant-encode-decode-roundtrip-10s.log

- Command: same shape, target `fuzz_quant_encode_decode_roundtrip`
- Result: 5,446 runs / 10s (495 exec/s), 25 new units, 0 crashes.
  Slower because each iteration runs `ProdQuantizer::new` +
  `encode` + `decode_approximate`; the property cares about
  per-run correctness, not throughput.

### fuzz-vector-normalize-structured-10s.log

- Command: same shape, target `fuzz_vector_normalize_structured`
- Result: 3,758 runs / 10s (341 exec/s), 11 new units, 0 crashes.

## Key result lines cited by request.md

- `Done N runs in 10 second(s)` + `stat::new_units_added: K` +
  zero crashes per target.
- Coverage numbers: topk 77k exec/s, quant 495 exec/s, vector
  normalize 341 exec/s.
- §Exit #1 closes: 4 of 9 fuzz targets use Arbitrary (parse_text,
  unpack_mse, vector_normalize, plus quant/topk new) — the
  remaining 4 are decoder targets that stay raw per §Why.

## Task 46 progress

3 of 5 §Exit gates closed (#1, #4, #5). #2 (SQLsmith) and #3
(Honggfuzz/AFL+) remain.
