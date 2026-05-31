# Task 70 / Packet 004: Frontier Neighbor Retention Slice

## Packet Scope

- Code commit: `dd42450f7fd0215d9c7385dd9cc1b25c0443b769`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Suite config: `artifacts/suite.json`
- Summary: `artifacts/phase2-frontier-summary.md`
- Normalized results: `artifacts/results.jsonl`

This packet requests review for the first Phase 2 P0 frontier/candidate-management slice.

## Code Change

`src/am/ec_diskann/scan.rs` now:

- moves each decoded tuple's existing neighbor vector into the queued frontier entry instead of collecting a second vector;
- stores `neighbor_count` separately and iterates only the filled prefix;
- keeps the sorted retained-best vector capped at `list_size` after every insert.

The change is intended to reduce allocation and retained-tail work inside the Phase 1 dominant frontier phase while preserving traversal order and candidate ranking semantics. It introduces no new `unsafe`.

## Validation

Commands and logs:

- `cargo fmt --check`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`
- `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file artifacts/install-ecaz-pg-test.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output artifacts/suite-manifest.json --results-output artifacts/results.jsonl --log-file artifacts/suite-run.log`

The scan unit module passed 18 tests. The full suite passed.

## Measurement

- Recall preserved: L=64 `0.9965`; L=200 `0.9975`.
- Latency changed from L=64 mean `0.65 ms` to `0.64 ms`, and L=200 mean `0.96 ms` to `0.91 ms`.
- pgvectorscale comparison now shows L=64 parity at `0.60 ms` mean, and L=200 `ec_diskann` at `0.77 ms` vs pgvectorscale `1.13 ms`.
- Frontier phase changed from `269.62 -> 263.60 us` at L=64 and `553.04 -> 527.90 us` at L=200.

## Reviewer Notes

The slice helps but does not retire the frontier P0. Frontier remains `70.77%` of profiled time at L=64 and `83.05%` at L=200 after the change, so follow-up candidate-management work is still justified.
