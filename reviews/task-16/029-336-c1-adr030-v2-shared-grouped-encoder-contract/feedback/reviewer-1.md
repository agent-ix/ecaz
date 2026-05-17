## Feedback: ADR-030 v2 Shared Grouped Encoder Contract

This closes the duplicated-encoder concern flagged on packets 311, 315, 317, 318,
and 333.

### What's right

- `src/quant/grouped_pq.rs::pack_grouped_pq_nibbles` is now the single source of
  truth for nibble packing. Both build-side `encode_grouped_pq` and study-side
  `encode_grouped_pq` route through it. Verified in code: both call the same helper.
- The helper is narrow (one function, one purpose) and has a direct assertion
  (`centroid_index < 16`) that catches the only way this codec can go wrong. Good
  defense.
- Even-count and odd-count packing both tested. Good coverage for an operation that
  has a 50% chance of having an off-by-one in the tail byte.
- Exported through `bench_api` rather than duplicated, so the binary doesn't have to
  carry a second copy for visibility reasons.

### Concerns

1. **Build-side and study-side still have separate nearest-centroid search.** The
   packet is explicit about this: "keep build-side nearest-centroid search local to
   `build.rs`." That's defensible — the two call sites may want different inner-loop
   shapes — but it does mean the "encode" pipeline is shared in only its last step.
   If training or centroid-index derivation ever diverges, the bug will look like
   "same packed bytes produced by different inputs." Worth adding a cross-crate
   test that asserts `derive_grouped_search_code_from_source` and the study-side
   derivation produce the same centroid index stream for the same input vector and
   the same model. Packer equality is necessary but not sufficient.

2. **`bench_api` surface.** Exporting the packer through `bench_api` means the public
   surface of that module now depends on the internal codec layout. That's fine
   today but if the binary format ever changes, `bench_api` versioning has to follow.
   Probably warrants a comment line on the export about it being codec-layout-
   coupled.

### Observation

The mechanical part of the duplication (packing) is resolved. The semantic part
(centroid index derivation) is next. Small packet, right size.
