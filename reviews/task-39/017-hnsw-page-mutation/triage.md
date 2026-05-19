# Mutation Triage

Target: `src/am/ec_hnsw/page.rs`

Harness: `make mutants MUTANTS_MODULE=src/am/ec_hnsw/page.rs MUTANTS_OUTPUT_DIR=... MUTANTS_JOBS=2`

## Initial Survivors

Initial run: `artifacts/page.rs.mutants/mutants.out/*`

Summary:

- Total: 477
- Missed: 119
- Caught: 283
- Timeout: 0
- Unviable: 75

Survivor groups:

- Metadata byte-size and offset arithmetic.
- Payload flag bit arithmetic, including shift-by-zero on the binary sidecar flag.
- Current-format metadata constructor codec gating for zero dimensions and zero bits.
- Metadata page/content minimum length and current-to-legacy fallback paths.
- Element, grouped-hot, and turbo-hot heap TID count bounds.
- Element/grouped/turbo encoded length and binary word count calculations.
- Neighbor tuple count, slot, encoded-length, and max-level fit boundaries.
- HNSW typed `DataPage` and `DataPageChain` update wrappers returning `Ok(())`.

## Rerun-1

Run: `artifacts/rerun-1/page.rs.mutants/mutants.out/*`

Summary:

- Total: 476
- Missed: 9
- Caught: 386
- Timeout: 6
- Unviable: 75

Disposition:

- The timeout mutants were caused by a test helper that searched for the maximum fitting element tuple length with a loop. The test was rewritten to assert the fixed default-page boundary directly.
- Remaining misses were metadata minimum/current-layout boundaries, zero-neighbor tuple decoding, and exact max-level fit behavior.

## Rerun-2

Run: `artifacts/rerun-2/page.rs.mutants/mutants.out/*`

Summary:

- Total: 444
- Missed: 1
- Caught: 369
- Timeout: 0
- Unviable: 74

Disposition:

- The remaining survivor skipped current-format metadata decoding for the exact minimum current metadata page. Added an exact-minimum current page decode assertion.

## Final

Run: `artifacts/final/page.rs.mutants/mutants.out/*`

Summary:

- Total: 444
- Missed: 0
- Caught: 370
- Timeout: 0
- Unviable: 74

No missed or timeout mutants remain in the final run.
