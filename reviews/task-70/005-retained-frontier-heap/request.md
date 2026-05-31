# Task 70 / Packet 005: Retained Frontier Heap Slice

## Packet Scope

- Code commit: `9bbef9ecf718bab30ef543b5f21d4728267136d0`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Previous frontier slice: `reviews/task-70/004-frontier-neighbor-retention/artifacts/phase2-frontier-summary.md`
- Suite config: `artifacts/suite.json`
- Manifest: `artifacts/manifest.md`
- Summary: `artifacts/phase2-retained-frontier-summary.md`
- Normalized results: `artifacts/results.jsonl`

This packet requests review for the second Phase 2 P0 frontier/candidate-management slice.

## Code Change

`src/am/ec_diskann/scan.rs` now:

- tracks retained best scan candidates in a bounded `BinaryHeap<ScanCandidate>`;
- uses the current worst retained candidate from that heap as the traversal stop threshold;
- sorts the retained heap only once before returning results.

This replaces per-candidate sorted-vector insertion and tail shifting while preserving output ordering and candidate ranking semantics. It introduces no new `unsafe`.

## Validation

Commands and logs:

- `cargo fmt --check`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`
- `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file artifacts/install-ecaz-pg-test.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output artifacts/suite-dry-run-manifest.json --log-file artifacts/suite-dry-run.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output artifacts/suite-manifest.json --results-output artifacts/results.jsonl --log-file artifacts/suite-run.log`

The scan unit module passed 18 tests. The full suite passed.

## Measurement

- Recall preserved: L=64 `0.9965`; L=200 `0.9975`.
- Latency vs packet 004: L=64 mean stayed `0.64 ms`; L=200 mean changed `0.91 -> 0.90 ms`; p95 stayed `0.73 ms` and `1.10 ms`.
- pgvectorscale comparison: L=64 `ec_diskann` mean `0.63 ms` vs pgvectorscale `0.61 ms`; L=200 `ec_diskann` mean `0.80 ms` vs pgvectorscale `1.16 ms`.
- Phase split vs packet 004: L=64 total `372.50 -> 366.32 us`, frontier `263.60 -> 261.37 us`; L=200 total `635.61 -> 641.93 us`, frontier `527.90 -> 531.88 us`.

## Reviewer Notes

This slice is semantics-preserving but not a clear performance win. It slightly improves L=64 phase timing and L=200 latency-table mean, but the L=200 raw profile worsens by about 1%. I do not consider this to retire the frontier P0; please review whether the simpler bounded-retained structure is worth keeping, or whether this slice should be shelved/reverted in favor of a larger frontier strategy.
