# Task 115 / 001 manifest

- Task bucket: `reviews/task-115/`
- Packet path: `reviews/task-115/001-residual-scoring-design/`
- Head SHA: `0225febd2df37b2de979f8e34c73e72547df5bcb`
- Branch: `task-115-ivf-rabitq-residual-quantization`
- Phase: 1 (scoring design + scalar reference tests)
- Host: dev/review box (no `ecaz` binary, no staged corpora). Scalar reference
  tests only; no benchmark numbers produced (NFR-007 — Phases 4/5 deferred).

## Artifacts

| file | what | command |
|------|------|---------|
| `phase1-residual-tests.log` | 4 residual scalar reference tests, all passed | `cargo test --no-default-features --features pg18 --lib quant::rabitq::tests::residual` |

## Key result lines cited by request.md

```
test quant::rabitq::tests::residual_code_is_byte_identical_shape_to_absolute_code ... ok
test quant::rabitq::tests::residual_estimate_recovers_exact_residual_term ... ok
test quant::rabitq::tests::residual_beats_absolute_on_concentrated_lists ... ok
test quant::rabitq::tests::residual_scoring_matches_exact_within_tolerance ... ok
test result: ok. 4 passed; 0 failed; ...
```

## Phase-1 stop-condition finding

Residual correction metadata = **0 extra bytes per posting** vs plain RaBitQ
(same code layout, same 12-byte scalar tail; centroid term is exact + per-list).
The stop condition does not fire; residual mode is index-size-neutral.

## Re-run

```
cargo test --no-default-features --features pg18 --lib quant::rabitq::tests::residual
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```
