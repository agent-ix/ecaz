# Task 97 Review Request: Local QJL32 Suite And HNSW QJL Registration

## Scope

This packet follows reviewer clarification that Task 97 targets the current production QJL surface: canonical TurboQuant `dim=1024,bits=4,seed=42`, not the no-QJL LUT32 `dim=1536` lane.

Code checkpoint: `70f6f2cf3c2f3c06a67139754242ce2c465d1f3e`

Changes:
- Routed HNSW TurboQuant production-QJL exact scoring through `QuantCodec::score_ip_batch`.
- Preserved the existing full-LUT no-QJL HNSW batch path by generalizing the exact-payload batch helper.
- Added HNSW graph helper variants that can score a loaded successor slice through a batch callback.
- Covered TurboQuant hot/cold HNSW storage by loading cold exact payloads into the same QJL batch queue.
- Added a local suite config for the clarified QJL fixture under `artifacts/task97-local-qjl32-suite.json`.

## Local Validation

Rust:
- `cargo fmt --check` (`artifacts/cargo-fmt-check.log`)
- `cargo test hnsw_turboquant_qjl_scan_codec_batch_uses_qjl32_path --lib -- --color never` (`artifacts/cargo-test-hnsw-qjl-codec.log`)
- `cargo test turboquant_exact_payload_batch --lib -- --color never` (`artifacts/cargo-test-turboquant-exact-payload-batch.log`)

Local PG18:
- `target/debug/ecaz ... dev install ecaz-pg-test --pg 18`
  - backend assertion passed
  - sha256 `041ce14cf789394aa9a91886d873fe11bcd1b35aae71a69dda1f320c0a54facb`
- `target/debug/ecaz ... bench suite run ... --only-tag kernel_on`

No GitHub CI or AWS runs were used.

## Evidence Summary

Current artifacts are documented in `artifacts/manifest.md`.

Direct `[block-kernel-counters]` evidence:
- IVF `turboquant_qjl` AVX2: `kernel_candidates=24096` at `nprobe=8`; `kernel_candidates=51200` at `nprobe=16`.
- SPIRE `turboquant_qjl` AVX2: `kernel_candidates=13696` at `nprobe=8`; `kernel_candidates=28800` at `nprobe=16`.
- HNSW `turboquant_qjl` scalar tails: `scalar_candidates=29763` at `ef_search=32`.

The HNSW m=8 fixture now emits direct `surface=hnsw quant=turboquant_qjl` rows. It does not hit AVX2 blocks because each graph expansion remains below block width 32, so the row is correctly attributed to `isa=scalar`.

## Reviewer Notes

Please review the HNSW hot/cold exact payload batching path in `src/am/ec_hnsw/scan.rs` and the graph-side batched score callback additions in `src/am/ec_hnsw/graph.rs`.
