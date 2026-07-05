# Task 70 / Packet 006: Shelved Rerank Result-Retention Slice

## Packet Scope

- Measured code commit: `4f499e27910399760f8c535588a8fdab805bc1b6`
- Shelve/revert commit: `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Previous retained-frontier packet: `reviews/task-70/005-retained-frontier-heap/artifacts/phase2-retained-frontier-summary.md`
- Suite config: `artifacts/suite.json`
- Manifest: `artifacts/manifest.md`
- Summary: `artifacts/phase2-rerank-retention-summary.md`
- Normalized results: `artifacts/results.jsonl`

This packet records a Phase 2 P0 rerank slice that was measured and shelved.

## Code Change Measured

`src/am/ec_diskann/scan.rs` briefly retained only the best `top_k` exact-rerank results in a bounded heap, instead of materializing all `rerank_budget` exact results and sorting/truncating at the end.

The change preserved rerank call count and heap-TID order, but it regressed the current `rerank_budget=64` / `top_k=10` path. It was reverted by `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`.

## Validation

Commands and logs:

- `cargo fmt --check`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`
- `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file artifacts/install-ecaz-pg-test.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output artifacts/suite-dry-run-manifest.json --log-file artifacts/suite-dry-run.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output artifacts/suite-manifest.json --results-output artifacts/results.jsonl --log-file artifacts/suite-run.log`
- post-revert `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-after-revert.log`

The scan unit module passed 18 tests before measurement and again after the revert. The full suite passed.

## Measurement

- Recall preserved: L=64 `0.9965`; L=200 `0.9975`.
- Latency vs packet 005: L=64 mean `0.64 -> 0.63 ms`, p95 `0.73 -> 0.74 ms`; L=200 mean `0.90 -> 0.96 ms`, p95 `1.10 -> 1.23 ms`.
- Phase split vs packet 005: L=64 exact rerank `84.58 -> 97.64 us`; L=200 exact rerank `88.69 -> 101.02 us`.

## Reviewer Notes

This is intentionally a shelving packet, not a keep request. The result-retention heap adds more overhead than it removes at the current budget. The branch has already been returned to the packet-005 scan implementation by revert commit `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`.
