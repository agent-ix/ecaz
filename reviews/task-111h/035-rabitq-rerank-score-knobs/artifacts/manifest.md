# Artifact Manifest

Packet: `reviews/task-111h/035-rabitq-rerank-score-knobs`

Task bucket: `reviews/task-111h`

Head SHA before packet commit:
`26de9a0c6a72a92646fe275d3883889008a82e58`

Created: `2026-06-20T20:38:25Z`

## Scope

Implementation checkpoint for the 024 feedback follow-up. This packet adds the
RaBitQ index-rerank score/clip levers needed for the next benchmark A/B. It does
not run the 100k benchmark suite.

## Artifacts

- `cargo-test-rabitq-rerank-options.log`
- `cargo-test-rabitq-index-rerank-levers.log`
- `cargo-check-pg18.log`

## Commands

```sh
cargo test --no-default-features --features pg18 rabitq_rerank --lib
cargo test --no-default-features --features pg18 rabitq_index_rerank_exposes_least_squares_and_clip_levers --lib
cargo check --no-default-features --features pg18
```

## Key Result Lines

- `cargo-test-rabitq-rerank-options.log`:
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2208 filtered out`.
- `cargo-test-rabitq-index-rerank-levers.log`:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2209 filtered out`.
- `cargo-check-pg18.log`:
  `Finished dev profile`.

## Benchmark Status

No benchmark was run in this packet. The new levers are intended for a follow-up
`ecaz bench suite` packet over the existing 100k Task 111h isolated surface.
