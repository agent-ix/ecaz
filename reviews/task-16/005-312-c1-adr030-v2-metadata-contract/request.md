# Review Request: C1 ADR-030 V2 Metadata Contract

## Context

Packet `311` kept the ADR-030 grouped-PQ feasibility spike:

- true grouped `PQ4` on transformed data is materially better than the rejected packet `280`
  reinterpretation
- the likely architecture remains `binary prefilter + grouped FastScan + tiny rerank`

The next narrow slice is to make the format contract explicit before touching tuple layout or scan
runtime.

## Problem

The metadata page still has no explicit index-format version or payload descriptors.

That is a problem for ADR-030 because the grouped-code redesign needs at least:

1. a way to distinguish current scalar-format indexes from future grouped-format indexes
2. a place to record transform kind
3. a place to record search-code and rerank payload descriptors

Without that contract, ADR-030 v2 remains underspecified at the persistence boundary.

## Planned Slice

Add a versioned metadata-page contract that:

1. preserves current v1 scalar-format behavior
2. records explicit format and payload descriptors
3. leaves room for the future grouped search-code v2 layout

This slice should stay at the metadata boundary only:

- no element tuple changes yet
- no runtime grouped search path yet
- no v2 builder yet

## Implementation

Implemented a versioned metadata contract in the page codec and threaded current-format writes
through it.

Metadata changes:

- added explicit `format_version` with:
  - `v1 = current scalar format`
  - `v2 = future grouped format`
- added transform descriptor:
  - `unknown`
  - `srht`
  - `opq`
- added search-code descriptor:
  - `unknown`
  - `scalar_quantized`
  - `grouped_pq`
- added rerank descriptor:
  - `none`
  - `scalar_quantized`
  - `grouped_pq`
- added payload flags for:
  - persisted binary sidecar
  - grouped search code
  - cold rerank payload
- added search-shape fields:
  - `search_bits`
  - `search_subvector_count`
  - `search_subvector_dim`

Compatibility behavior:

- metadata decode now accepts both:
  - legacy metadata-page payloads
  - new versioned metadata-page payloads
- legacy pages decode as explicit `v1 scalar` metadata
- metadata page reads try the new special-area size first, then fall back to the legacy size
- metadata page rewrites now reinitialize the page with the required special-area size before
  copying metadata bytes, so legacy pages can safely grow to the new contract on rewrite

Current-format plumbing:

- current build-time metadata creation now goes through a single
  `MetadataPage::current_v1_scalar(...)` constructor
- build finalization records whether the persisted binary sidecar is present in current v1 indexes
- tests, proptests, and page-codec benchmarks now construct current-format metadata through the new
  explicit v1 contract

Non-ADR change required for green validation:

- corrected a stale `approx_score_study` grouped-PQ nibble-packing test expectation from `0x31` to
  `0x21`; this was exposed by the required `cargo test` / `cargo pgrx test` checkpoint and was not
  caused by the metadata work

## Measurements

This packet is a format-contract slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test metadata_roundtrip --lib`: passed
- `cargo test metadata_decode_page_accepts_legacy_layout --lib`: passed
- `cargo test prepared_query_cache_lifetime_tracks_scan_state --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed after correcting the stale grouped-PQ bin-test expectation
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

The metadata persistence boundary now has enough structure to support ADR-030 v2 without guessing at
format semantics later.

What this de-risks:

1. future grouped-format indexes can be distinguished from current scalar indexes explicitly
2. transform/search/rerank choices now have durable on-disk descriptors
3. metadata-page reads remain backward-compatible with already-written legacy pages

What this does not do:

1. no v2 tuple layout yet
2. no grouped runtime scorer yet
3. no builder path for grouped payloads yet

## Next Slice

The next narrow slice should define the hot/cold tuple payload contract for v2:

1. hot search payload for binary sidecar plus grouped search code
2. cold rerank payload placement and descriptor semantics
3. page-local layout invariants needed for scan-time locality
