## Feedback: ADR-030 v2 Metadata Contract

Read the code in `src/am/page.rs` around the `MetadataPage` fields, `TransformKind`,
`SearchCodecKind`, `RerankCodecKind`, and `GraphStorageFormat` enums.

### What's right

- Adding `format_version`, `transform_kind`, `search_codec_kind`, `rerank_codec_kind`,
  `payload_flags`, `search_subvector_count`, `search_subvector_dim` as separate fields
  rather than packing them into flag bits keeps the contract readable and lets legacy
  v1 indexes fall through `MetadataPage::current_v1_scalar(...)` without code changes.
- Keeping `V1_SCALAR = 1` and `V2_GROUPED = 2` as distinct version numbers (not a single
  "feature bit") means a future v3 does not have to retrofit around a bitmask.

### Concern: `search_bits` vs `bits`

`rerank_code_len` is derived from `metadata.bits`, not `search_bits`. That works today
because v1 scalar uses `bits` for the scalar code and v2 uses `bits` for the rerank code
while `search_bits` carries the PQ4 width. But the names suggest two independent fields,
and if a v3 ever diverges rerank bit width from scalar-v1 bit width, the collision will
be silent.

Two options to consider:
1. Rename: `bits` → `rerank_bits` (or `scalar_bits`) now while there's only one caller.
2. Add an assertion that for `V2_GROUPED`, `bits == rerank_bits` at metadata validation
   time.

Not blocking this packet. Flagging it because the seam is being set here and reshaping
later is harder.

### Legacy compatibility

The `current_v1_scalar(...)` constructor is the right shape. Just make sure any future
migration check (e.g. amcheck-style) reads format_version first, not payload_flags, so
a legacy v1 page with default-zero new fields is not misinterpreted.
