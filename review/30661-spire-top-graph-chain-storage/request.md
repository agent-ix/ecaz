# Review Request: SPIRE Top-Graph Chain Storage

Phase 9.2 removes the single-tuple top-graph storage ceiling by promoting the
existing relation-object V2 chain format from routing-only to generic
partition-object storage for root, internal, and top-graph objects.

Code checkpoint: `64fb3324` (`Enable chained SPIRE top graph storage`)

## Scope

- Replaces the top-graph single-tuple rejection path with chained relation
  object writes when the encoded graph exceeds one page tuple.
- Renames the routing-specific relation chain codec helpers and flags to
  generic partition-object chain names.
- Extends chain meta/segment decode validation to accept `TopGraph` objects
  while continuing to reject unsupported object kinds.
- Makes `read_top_graph_object`, `read_object_header`,
  `read_object_bytes`, and active tuple locator discovery understand chained
  top-graph objects.
- Adds top-graph snapshot diagnostics:
  - `object_tuple_count`
  - `object_segment_count`
- Marks Task 30 Phase 9.2 complete in the detailed and summary task files.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 relation_object_chain_codecs_accept_top_graph_kind --lib`
- `cargo test --no-default-features --features pg18 top_graph --lib`
- `cargo test --no-default-features --features pg18 large_top_graph --lib`

The new `large_top_graph` pgrx test builds a top graph large enough to require
the V2 chain path, then asserts `object_bytes > 8192`,
`object_tuple_count > 1`, `object_segment_count > 0`, and
`object_tuple_count = object_segment_count + 1`.

## Review Focus

- Check that reusing the relation-object V2 chain format for top graphs keeps
  manifest/placement/epoch validation intact.
- Check whether `read_object_header` returning the chain-meta header flags for
  chained top graphs is consistent with the existing chained routing-object
  behavior.
- Check whether the new snapshot diagnostics are sufficient for later Phase 9
  and Phase 10 performance work, or whether they should also report the chain
  meta tuple block/segment tuple blocks separately.
