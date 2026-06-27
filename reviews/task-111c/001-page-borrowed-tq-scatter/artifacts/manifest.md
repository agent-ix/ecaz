# Task 111c Packet 001 Artifact Manifest

- head SHA: `11b145d2d3af51106dae03b34c0dac7cccc5d8d8`
- task bucket: `reviews/task-111c`
- packet path: `reviews/task-111c/001-page-borrowed-tq-scatter`
- timestamp: `2026-06-17T14:11:08-07:00`
- lane / fixture / storage format / rerank mode: local compile/unit validation, no corpus fixture, no benchmark lane
- isolated one-index-per-table or shared-table surface: not applicable; no SQL benchmark/load run

## Artifacts

### `artifacts/cargo-test-columnar-single-page-range.log`

- command:

```sh
script -q -c "cargo test columnar_single_page_range --no-default-features --features pg18" reviews/task-111c/001-page-borrowed-tq-scatter/artifacts/cargo-test-columnar-single-page-range.log
```

- purpose: focused PG18-feature compile and unit validation for the new columnar logical-byte to raw-page mapper used by the pinned-page scan path.
- key result lines:

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2128 filtered out; finished in 0.00s
```
