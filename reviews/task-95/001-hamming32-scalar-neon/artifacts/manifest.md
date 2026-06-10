# Manifest: Task 95 Packet 001 hamming32 Scalar + NEON

- Head SHA: `4a67d05b0`
- Task bucket: `reviews/task-95/`
- Packet path: `reviews/task-95/001-hamming32-scalar-neon/`
- Branch: `task-93-rabitq-block-kernel` (depends on Task 93 packet 004/005
  infra; see request.md)
- Lane: local M5 (aarch64/NEON) code slice; bench cells deferred to packet
  002 (see request.md §Deferred)
- Isolation: not applicable (no bench fixtures in this packet)

## Artifacts

### `cargo-test-hamming32.log`

- `cargo test -p ecaz --lib --no-default-features --features pg18 hamming32 -- --test-threads=1`
- 3 passed: block32 integer-exact vs scalar reference across word counts
  {1,3,12,24,25} (odd-word tails included), partial integer-exact across
  counts {1,2,7,22,31}, scalar reference is XOR+popcount with
  self-distance 0. Real NEON exercised on this host.

### `cargo-test-diskann-quantizer.log`

- 14 passed, including
  `diskann_binary_sidecar_prefilter_batch_is_exactly_per_candidate`.

### `cargo-test-candidate-batch.log`

- 10 passed.

### `cargo-clippy.log`

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — clean.
