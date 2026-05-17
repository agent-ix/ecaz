# Review Request: Task 28 IVF Candidate Dedup Pool

## Summary

This packet addresses packet 30047 feedback seq 02 item 5: probe-candidate
aggregation allocated a fresh `HashMap<heap_tid, candidate>` for every scan.

Implemented:

- Added a scan-opaque candidate dedup map pointer.
- Allocated the map once per scan descriptor and reused it across rescans.
- Cleared and reserved the map for each probe materialization instead of
  constructing a fresh `HashMap`.
- Kept the existing heap-tid dedup semantics, tie-breaking, output sort order,
  rerank path, and MVCC behavior unchanged.
- Freed the pooled map in `amendscan`.

## Scope

This is allocation-pressure work only. It introduces no new recall, latency, or
throughput claim. DiskANN remains task 29.

## Validation

- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - `6 passed; 0 failed`
- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `77 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`

## Next

Re-run the DBPedia 10k/25k `nlists x nprobe x rerank_width` sweep against the
current optimized cost profile. Then complete the deeper build/training/vacuum
read before making product-benchmark statements.
