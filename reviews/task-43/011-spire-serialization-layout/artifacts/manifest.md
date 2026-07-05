# Task 43 Packet 011 Artifact Manifest

Head SHA: `e555e3c43c2aee809a2a46d9f6885a507c833bd5`

Task bucket: `reviews/task-43/011-spire-serialization-layout`

Timestamp: `2026-05-18T13:29:23-07:00`

Surface: pure Rust SPIRE storage serialization/layout helpers. No PostgreSQL
table, index, storage format fixture, or rerank runtime table was created;
isolated/shared table distinction is not applicable.

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-spire-assignment-rows.log` | `cargo +nightly miri test --lib miri_assignment_` | 7 passed; 0 failed; assignment row round-trip, prefix/tail decoding, borrowed ref decoding, visibility helpers, invalid flags, invalid payload format, and length mismatch. |
| `miri-spire-delta-object.log` | `cargo +nightly miri test --lib miri_delta_partition_object` | 5 passed; 0 failed; delta object insert/delete round-trip and invalid header, flags, delete payload, and duplicate vec-id rejection. |
| `miri-spire-vec-id-invalid.log` | `cargo +nightly miri test --lib miri_vec_id` | 1 passed; 0 failed; invalid vec-id shapes rejected. |
| `miri-spire-local-vec-id.log` | `cargo +nightly miri test --lib miri_local_vec_id` | 1 passed; 0 failed; local vec-id sequence round-trip. |
| `miri-spire-global-vec-id.log` | `cargo +nightly miri test --lib miri_global_vec_id` | 1 passed; 0 failed; global vec-id payload preservation. |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | Exit 0; rustfmt emitted existing unstable-option warnings. |
| `git-diff-check.log` | `git diff --check` | Exit 0; no whitespace errors. |

## Coverage Notes

- This packet promotes existing bounded pure storage tests instead of creating
  duplicate fixtures.
- Together with prior storage Miri tests for page, DiskANN, HNSW, SPIRE leaf V2,
  and top graph formats, this closes the remaining SPIRE delta / assignment /
  vec-id serialization row in the campaign tracker.
