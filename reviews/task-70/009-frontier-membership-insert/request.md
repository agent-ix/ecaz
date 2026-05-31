# Task 70 / Packet 009: Frontier Membership Insert

## Packet Scope

- Code commit: `31de2206a6eda1578d48d90e70661eaf24108fda`
- Review drivers:
  - `reviews/task-70/008-frontier-subtiming-profile/request.md`
  - `reviews/task-70/008-frontier-subtiming-profile/artifacts/frontier-subtiming-summary.md`
- Manifest: `artifacts/manifest.md`
- Summary: `artifacts/membership-insert-summary.md`

This packet requests review for the first post-subtiming frontier slice. It is intentionally narrow: remove a duplicate membership lookup for first-seen neighbors in the DiskANN scan loop.

## Code Change

`src/am/ec_diskann/scan.rs` now uses `scratch.in_frontier.insert(nbr)` as the neighbor membership test. Previously the loop called `contains(&nbr)` and then `push_frontier_entry` inserted the same TID for new neighbors. `push_frontier_entry` now only pushes an already-marked candidate into the heap.

Expected behavior is unchanged:

- duplicate neighbors are still skipped;
- first-seen neighbors are still marked before heap insertion;
- tombstoned/stripped candidates still expand but cannot consume retained result slots;
- rerank order and final result ordering are unchanged.

No new `unsafe` was introduced.

## Validation

Commands and logs:

- `cargo fmt --check` -> `artifacts/cargo-fmt-check.log`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`
- `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file artifacts/install-ecaz-pg-test.log`
- `./target/debug/ecaz bench suite run --config artifacts/suite.json ...` -> `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`

The focused scan module passes 19 tests. PG18 cargo check and fmt check both finish successfully. The suite run succeeded with packet-local artifacts.

## Measurement Results

Recall is unchanged:

| lane | packet 008 recall | packet 009 recall | packet 009 q-time |
| --- | ---: | ---: | ---: |
| L64 | 0.9965 | 0.9965 | 0.66 ms |
| L200 | 0.9975 | 0.9975 | 0.90 ms |

pgvectorscale compare step:

| lane | packet 008 ec mean | packet 009 ec mean | delta | packet 008 ec p95 | packet 009 ec p95 | delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| L64 | 0.69 ms | 0.66 ms | -4.3% | 0.82 ms | 0.80 ms | -2.4% |
| L200 | 0.97 ms | 0.91 ms | -6.2% | 1.14 ms | 1.08 ms | -5.3% |

Profiled frontier means:

| lane | packet 008 frontier mean | packet 009 frontier mean | delta | packet 008 total mean | packet 009 total mean | delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| L64 | 401.60 us | 370.09 us | -7.8% | 513.30 us | 481.90 us | -6.1% |
| L200 | 920.04 us | 842.88 us | -8.4% | 1037.58 us | 960.47 us | -7.4% |

The sub-bucket comparison needs one caveat: packet 009 moved membership insertion timing into `frontier_visited_set_us`, so `frontier_candidate_heap_us` and `frontier_visited_set_us` are not apples-to-apples with packet 008. The enclosing `frontier_us`, `total_us`, recall, and compare-step latency are the stable review signals.

## Reviewer Notes

This looks like a modest keeper slice: the code is simpler, recall is unchanged, compare latency improves at both L64 and L200, and the profiled frontier residual moves in the expected direction. I am not treating it as Task 70 closeout; further P0 work still needs reviewer feedback on packet 008/009 and then either another accepted frontier/rerank slice or a documented shelve decision.
