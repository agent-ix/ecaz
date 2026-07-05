# Task 43 Packet 008 Artifact Manifest

Head SHA: `2a4d09c97132ca685342a149a019f3a923a56ed2`

Task bucket: `reviews/task-43/008-breadth-closure-existing-tests`

Timestamp: `2026-05-18T12:54:01-07:00`

Surface: pure Rust Miri/careful unit coverage. No PostgreSQL table, index, storage
format fixture, or rerank runtime table was created; isolated/shared table
distinction is not applicable.

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-diskann-build.log` | `cargo +nightly miri test --lib miri_build_` | 2 passed; 0 failed; 1792 filtered; bounded Vamana build stats and pass-1 extra candidates. |
| `miri-diskann-vacuum.log` | `cargo +nightly miri test --lib miri_vc_` | 8 passed; 0 failed; 1786 filtered; mark/delete/strip/fully-dead/repair/state-machine coverage. |
| `miri-hnsw-beam.log` | `cargo +nightly miri test --lib miri_beam_search` | 4 passed; 0 failed; 1790 filtered; dedupe, stale leaders, fully stale frontier, forget/reseed. |
| `miri-hnsw-visible-frontier.log` | `cargo +nightly miri test --lib miri_visible_frontier` | 2 passed; 0 failed; 1792 filtered; scheduler preference and select-next refill. |
| `miri-spire-routed-rank.log` | `cargo +nightly miri test --lib miri_rank_routed` | 2 passed; 0 failed; 1792 filtered; bounded vec-id dedupe and primary tie-break. |
| `miri-spire-scored-candidate.log` | `cargo +nightly miri test --lib miri_scored_candidate` | 1 passed; 0 failed; 1793 filtered; newer epoch then primary-role tie order. |
| `miri-spire-rerank.log` | `cargo +nightly miri test --lib miri_rerank_scored` | 3 passed; 0 failed; 1791 filtered; rerank prefix replacement, invisible drop, non-finite rejection. |
| `miri-spire-candidate-cursor.log` | `cargo +nightly miri test --lib miri_scan_candidate_cursor` | 1 passed; 0 failed; 1793 filtered; candidate cursor emits ranked candidates once. |
| `miri-spire-routing.log` | `cargo +nightly miri test --lib miri_route_` | 9 passed; 0 failed; 1785 filtered; root/internal/top-graph/recursive routing plus rejection paths. |
| `careful-harness-cargo-test.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | 69 passed; 0 failed; path-lifted DiskANN/HNSW/careful harness remains green. |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | Exit 0; rustfmt emitted existing unstable-option warnings. |
| `git-diff-check.log` | `git diff --check` | Exit 0; no whitespace errors. |

## Coverage Notes

- The existing full-size DiskANN Vamana build tests remain ordinary Rust tests
  because they are too slow for the aggregate Miri prefix. Packet 008 adds
  bounded 16-node and 12-node Miri variants that exercise the same production
  build helpers.
- The careful harness count increased from 67 to 69 because the new bounded
  Vamana Miri tests are path-lifted through `hardening/careful`.
- SPIRE scan and routing Miri tests are not yet mirrored by cargo-careful; the
  tracker keeps that as an open G6 blocker pending extraction/path-lift work.
