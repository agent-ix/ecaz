# Manifest: Task 93 Packet 005 AVX2 Backend + SVE Routing

- Head SHA: `4872107d8` (code commit `2726b8b4a` rebased onto the packet-004
  approval)
- Task bucket: `reviews/task-93/`
- Packet path: `reviews/task-93/005-avx2-backend-sve-routing/`
- Lane: local M5 (aarch64/NEON) validation of a code-only slice; AVX2 and
  SVE measurement deferred per `request.md` §Deferred
- Isolation: not applicable (no bench fixtures in this packet)

## Artifacts

### `cargo-test-rabitq32.log`

- `cargo test -p ecaz --lib --no-default-features --features pg18 rabitq32 -- --test-threads=1`
- 6 passed (host-generalized ISA expectations; NEON paths exercised here).

### `cargo-test-candidate-batch.log`

- Focused `am::common::candidate_batch` run: 10 passed.

### `cargo-clippy.log`

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — clean.
