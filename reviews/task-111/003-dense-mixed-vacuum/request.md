# Task 111 Review Request: Dense Mixed Scan and Vacuum Correctness

## Scope

This packet reviews code checkpoint `26e3f443dd5f` (`Task 111: cover dense mixed scans and vacuum`).

It builds on packet 002's gated dense block implementation by adding pg-level correctness coverage and minimum lifecycle handling for build-time dense blocks.

## Change

- Adds a fixed-size deleted bitmap to dense posting block tuples.
- Skips deleted dense postings during scan.
- Extends IVF vacuum rewrite to process row postings and dense posting blocks in the same list block range.
- Marks dense entries deleted in place during bulkdelete without changing tuple size or line pointers.
- Adds test-only scan counter snapshots for pg fixtures.
- Adds PG18 fixtures for:
  - gated dense build scan over build-time rows,
  - mixed dense block plus row-shaped live insert scan,
  - vacuum removal of a build-time dense posting.

## Validation

Packet-local artifacts:

- `artifacts/cargo-check-lib.log`
- `artifacts/cargo-test-ivf-explain.log`
- `artifacts/cargo-test-dense-posting.log`

Results:

```text
cargo check -q --lib
```

exited successfully.

```text
cargo test -q ivf_explain --lib
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2105 filtered out

cargo test -q dense_posting --lib
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out
```

## Remaining Task 111 Work

- Run the required `ecaz bench suite` evidence for TurboQuant and RaBitQ latency/recall/storage/build-time comparison before any promote/iterate/abandon recommendation.
- Add any benchmark-driven adjustments needed by that evidence.
