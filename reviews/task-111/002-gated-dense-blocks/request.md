# Task 111 Review Request: Gated Dense Posting Blocks

## Scope

This packet reviews code checkpoint `a82983bcac9d` (`Task 111: add gated IVF dense posting blocks`).

This is the first implementation slice after packet 001's counter instrumentation. It is intentionally still gated and does not promote the layout to default.

## Change

- Adds `dense_posting_blocks = 1` as an experimental `ec_ivf` reloption. The default remains off.
- Adds a dense posting block tuple codec with page-local scan arrays:
  - `gamma[count]`
  - `heap_tid_count[count]`
  - `heap_tid_offset[count]`
  - `rerank_tid[count]`
  - `heap_tids[total_heap_tids]`
  - `payloads[count * payload_stride]`
- Adds a build-time writer for frozen dense blocks when the gate is enabled for `auto`, `turboquant`, or `rabitq` IVF storage formats.
- Keeps insert/bootstrap paths on row-shaped postings.
- Adds mixed scan visitation so the same selected list block range can contain row postings and dense block tuples.
- Routes dense block payload arrays through the existing batch scorer and falls back to scalar scoring if a batch path is unavailable.
- Extends IVF EXPLAIN counters with:
  - `Row Postings Visited`
  - `Dense Blocks Visited`
  - `Dense Postings Visited`

## Validation

Packet-local artifacts:

- `artifacts/cargo-check-lib.log`
- `artifacts/cargo-test-dense-posting.log`
- `artifacts/cargo-test-ivf-explain.log`

Results:

```text
cargo check -q --lib
```

exited successfully.

```text
cargo test -q dense_posting --lib
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out

cargo test -q ivf_explain --lib
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out
```

## Known Remaining Task 111 Work

- Add pg-level controlled fixture coverage for gated dense builds and mixed dense+row scans.
- Add delete/vacuum lifecycle handling or a documented gate restriction before using dense blocks under churn.
- Run the required `ecaz bench suite` packet for TurboQuant and RaBitQ latency/recall/storage/build-time evidence before any promote/iterate/abandon recommendation.
