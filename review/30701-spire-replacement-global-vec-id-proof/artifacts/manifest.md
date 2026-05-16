# Artifact Manifest: 30701 SPIRE Replacement Global Vec-ID Proof

Head SHA: `8b8b2afaaa4631b5fa10f6662bcb0e6a1ffed6aa`
Packet: `review/30701-spire-replacement-global-vec-id-proof`
Timestamp: `2026-05-09T19:48:35-07:00`

## Scope

- Lane: Task 30 Phase 11.2 writer-side global vector identity.
- Fixture: local unit tests.
- Storage format: Leaf V2 fixed-width global vec IDs and row-encoded delta
  assignments.
- Rerank mode: not a rerank measurement packet.
- Surface: scheduled replacement row collection, merge replacement materializer,
  split replacement boundary fanout, and remote merge namespace tests.
- Index isolation: not a benchmark packet; no shared-table or multi-index
  performance claim.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo fmt` | Passed; existing rustfmt unstable import-grouping warnings printed |
| `cargo test global_vec_ids --lib` | Passed: 7 passed, 0 failed |
| `cargo test local_vec_ids_by_node --lib` | Passed: 2 passed, 0 failed |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `cargo test global_vec_ids --lib`: `test result: ok. 7 passed; 0 failed`.
- `cargo test local_vec_ids_by_node --lib`: `test result: ok. 2 passed; 0 failed`.
- `git diff --check` produced no output and exited successfully.
