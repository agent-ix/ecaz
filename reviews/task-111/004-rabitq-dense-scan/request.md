# Task 111 Review Request: RaBitQ Dense Scan Coverage

## Scope

This packet reviews code checkpoint `e2e8383916ad` (`Task 111: add RaBitQ dense scan coverage`).

It follows packet 003 by adding the missing RaBitQ-focused pg-level correctness fixture for gated dense posting blocks.

## Change

- Adds a PG18 fixture that builds an `ec_ivf` index with:
  - `storage_format = 'rabitq'`
  - `quant_bits = 1`
  - `dense_posting_blocks = 1`
- Verifies the scan returns all build-time rows.
- Verifies test-only scan counters report dense block traversal rather than row posting traversal:
  - `row_postings_visited = 0`
  - `dense_blocks_visited = 1`
  - `dense_postings_visited = 4`

## Validation

Packet-local artifacts:

- `artifacts/cargo-check-lib.log`
- `artifacts/cargo-test-dense-posting.log`

Results:

```text
cargo check -q --lib
```

exited successfully.

```text
cargo test -q dense_posting --lib
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out
```

## Remaining Task 111 Work

- Address the dense scan allocation/performance gap called out in packet 002 feedback, or explicitly carry it into benchmark interpretation.
- Run the required `ecaz bench suite` evidence for TurboQuant and RaBitQ latency/recall/storage/build-time comparison before any promote/iterate/abandon recommendation.
- Add any benchmark-driven adjustments needed by that evidence.
