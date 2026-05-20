# Task 50 Review Request: Unsafe Facade Contract Follow-Up

## Summary

This packet addresses reviewer feedback from the Task 50 unsafe facade packets for HNSW scan debug, IVF scan, SPIRE snapshots, and HNSW production scan accessors.

Code commits:

- `5878b6e3 Restore unsafe facade contracts`
- `3051e991 Clean HNSW facade helper lifetimes`

The follow-up preserves the Task 50 reduction targets while making the facade contracts explicit at the call boundaries reviewers flagged.

## Changes Under Review

- HNSW scan debug facade helpers that project from `IndexScanDesc` raw state are now `unsafe fn`, forcing callers to acknowledge the scan descriptor and opaque-layout contract.
- IVF debug scan helpers are now `unsafe fn`.
- IVF scan storage free helper now takes the owning `EcIvfScanOpaque` plus a slot selector, frees only a scan-owned slot, and clears that slot internally instead of accepting an arbitrary raw pointer.
- SPIRE snapshot live relation helpers are now `unsafe fn`, making the relation-pointer lifetime contract explicit at the call site.
- HNSW production scan opaque helpers are now `unsafe fn`.
- HNSW boxed pointer helpers now tie their output lifetime to an `&TqScanOpaque` / `&mut TqScanOpaque` borrow, avoiding an unconstrained lifetime projection.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Prior packet count | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/scan.rs` | 226 | 157 | 158 | <=158 | met |
| `src/am/ec_hnsw/scan_debug.rs` | 356 | 129 | 135 | <=249 | met |
| `src/am/ec_ivf/scan.rs` | 102 | 69 | 69 | <=71 | met |
| `src/am/ec_spire/coordinator/snapshots.rs` | 62 | 41 | 42 | <=43 | met |

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/scan.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_ivf/scan.rs src/am/ec_spire/coordinator/snapshots.rs'` passed.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_ivf/scan.rs src/am/ec_spire/coordinator/snapshots.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` still fails on the existing repo-wide clippy backlog. After the final code commit, searching the clippy artifact for the touched facade files and helper names returned no matches.

No benchmark result is claimed in this packet. This is a soundness-contract follow-up to prior facade slices and does not change scoring, traversal, or storage-format behavior.
