## Feedback: Core Storage-Format Naming

Read `page::GraphStorageFormat` at `src/am/page.rs:99-178` and the
metadata-decode branches at `:1409-1433`.

### What's right

- **Completes the rename one layer down from packet 380.**
  `page::GraphStorageFormat::{TurboQuant, PqFastScan}` now matches
  `graph::GraphStorageDescriptor::{TurboQuant, PqFastScan}`. The
  awkward "retranslate through old names" step in `graph.rs` is gone.
- **Wire bytes preserved.** `INDEX_FORMAT_V1_SCALAR` and
  `INDEX_FORMAT_V2_GROUPED` are still the disk-version constants.
  The decode path at `page.rs:174-175` maps them to the renamed
  enum variants but doesn't touch the actual byte values. That is
  exactly the "runtime types rename, wire format does not" contract
  from ADR-032.
- **Helper/test name cleanup is scoped.** `grouped_v2_metadata →
  pq_fastscan_metadata`, `experimental_grouped_v2_exact_traversal →
  pq_fastscan_exact_traversal`, and similar. Core AM surface is
  normalized without taking on the wider `src/lib.rs` pg-test rename
  (that lands in 388).

### Concerns

1. **Wire-tag constant names are now slightly misleading.** The
   runtime enum is `PqFastScan` but the wire tag is still
   `INDEX_FORMAT_V2_GROUPED`. A future reader might expect the wire
   tag to follow the rename. The right answer is to keep the wire
   tag fixed (renaming it would be a format break), but a brief
   comment near the constant declarations saying "these names
   describe the disk format version, not the product format name"
   would save future confusion.

2. **Splitting 380+384 created a temporary translation step.** Not
   a blocker — the tree was compilable and clippy-clean at each
   point — but the combined diff would have been smaller and more
   reviewable. Lesson for future renames: when a name crosses
   layers, one packet per layer is *less* reviewable than one packet
   for the whole cascade.

3. **Linker gap.** This is a pure-rename packet so the risk of
   functional regression is minimal. Still, every pg test in the
   arc relies on the renamed enum, and none ran locally.

### Observation

With 380+384 landed, the branch finally speaks one language. That
unblocks reviewer mental overhead on every subsequent packet.
