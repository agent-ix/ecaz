# Review Request: Leaf V3 Summary Storage

Code commit under review: `1d05f4f1d5ca78e2c6991502df4640a999bd0b5d`

This checkpoint adds the storage scaffold needed for Task 79 leaf-local block pruning. It does not yet claim a latency improvement: scanners still need to consume these summaries in a follow-up checkpoint.

## What Changed

- Added leaf object format V3 as a byte-compatible extension over leaf V2 metadata.
- V3 leaf metadata records `summary_block_rows`, `summary_count`, `first_summary_segment_locator`, and `summary_bytes_total`.
- Added V3 summary segment encoding/decoding for contiguous leaf row-block summaries.
- Added local and relation store insert/read paths for leaf rows plus block summaries.
- Preserved V2 behavior: empty or missing summaries stay on the existing V2 path, so old leaves fall back to full leaf scan behavior.
- Treat malformed V3 summary metadata or broken summary chains as decode/read errors rather than silently falling back.
- Extended relation-store active tuple locator enumeration to include V3 summary segment tuples.

The new unit coverage uses RaBitQ payload format for the V3 path, matching the Task 79 primary/default target. TurboQuant remains a comparison target for later measurement, not this storage checkpoint.

## Cost Model And Next Step

The starting block-size range remains 32, 64, and 128 rows. At 64 rows per summary, summary scoring overhead is approximately `1/64 = 1.56%` of row scoring if one summary is scored per block. To move the observed no-format-change surface from about 5.25M candidates to a strong <=4.0M target, block pruning needs to avoid about 1.25M row scores, roughly 24%, while keeping recall at or above the current Task 79 target band.

Next implementation checkpoint: materialize RaBitQ block summaries during leaf build, add a scanner-side block selector, and then run `ecaz bench suite` for the 32/64/128 row-block sweep before considering any TurboQuant comparison.

## Validation

See `artifacts/manifest.md`.

- `cargo check -p ecaz`: pass
- `cargo test -p ecaz leaf_partition_object_v3`: pass, 2 V3 storage tests
- `cargo test -p ecaz leaf_partition_object_v2_store_segments_large_leaf`: pass, V2 regression
