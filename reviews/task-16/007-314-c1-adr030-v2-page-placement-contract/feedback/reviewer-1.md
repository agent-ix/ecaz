## Feedback: ADR-030 v2 Page Placement Contract

Read the `DataPage` / `DataPageChain` helpers in `src/am/page.rs` for
`insert/read/update_grouped_hot` and `insert/read/update_rerank`.

### What's right

- Reusing the existing DataPage chain infrastructure for both hot and rerank tuples
  means grouped-v2 does not invent a parallel free-space tracking lane. Page placement
  mistakes in v2 will fail loudly instead of silently diverging from v1's layout
  invariants.
- Separate insert/read/update helpers per tag keep the dispatch explicit. No
  tag-polymorphism buried inside a single tuple accessor.

### Worth verifying before the scorer lands

1. Hot-tuple and rerank-tuple writes don't need to land on the same page, but the
   reranktid has to survive page placement decisions. Make sure there is a test that
   writes a grouped hot tuple and its rerank tuple to different DataPages and then
   reads the rerank via the hot tuple's reranktid. This is the cross-page contract the
   cold rerank fetch will depend on.
2. Placement order matters during build: rerank tuple has to be written before hot
   tuple so the hot tuple can record a valid reranktid. Confirm the build path in
   packet 315+ does this in that order, and that a build that fails between the two
   writes cannot leave orphan rerank tuples. A build-time panic test covering that
   would be valuable.

### Not in this packet's scope but adjacent

Vacuum will need to track both tuple tags. It does not today (vacuum.rs uses
`TqElementTuple::decode` only). Flag to track before grouped-v2 leaves the experimental
gate.
