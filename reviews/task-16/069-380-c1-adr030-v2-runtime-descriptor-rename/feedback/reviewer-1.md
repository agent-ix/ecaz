## Feedback: Runtime Descriptor Rename

Read `GraphStorageDescriptor` in `src/am/graph.rs:19-24` and the match
sites in `insert.rs`, `vacuum.rs`, `scan.rs`, `scan_debug.rs`.

### What's right

- **Rename is clean and mechanical.** `ScalarV1 → TurboQuant`,
  `GroupedV2(GroupedGraphLayout) → PqFastScan(PqFastScanLayout)`. No
  behavior change, and the variant struct carries its layout data
  through the rename intact. 18+ match sites (the same count flagged
  in task 15 as advisory audit work) all updated.
- **Wire tags kept stable.** `INDEX_FORMAT_V1_SCALAR` and
  `INDEX_FORMAT_V2_GROUPED` are still the on-disk version bytes. Task
  15 explicitly calls out "Wire tags are not renamed — they stay as
  disk versioning bytes" and that rule is honored here. Good
  separation of "product name" from "format version byte."
- **Error strings updated in the same packet.** `tqhnsw aminsert does
  not support PqFastScan indexes yet`, `PQ_FASTSCAN_*` constants,
  `tqhnsw PqFastScan metadata ...` — users no longer see any
  `grouped-v2` in runtime-visible strings from scan/insert/vacuum.

### Concerns

1. **Lower `page::GraphStorageFormat` still at `ScalarV1` /
   `GroupedV2` after this packet.** Packet 384 catches this up, but
   the two-packet split means `graph.rs` temporarily has to rewrap
   old names into new names. If reviewers read these packets in
   isolation they may think there's dead translation code; worth a
   one-line note that 384 finishes the job at the page layer.

2. **Out-of-scope comment on `page.rs` is right but understated.**
   The `page::GraphStorageFormat` enum *is* the disk-versioning
   layer, so renaming it is purely cosmetic from a wire perspective.
   Still, combining 380+384 into one slice would have been simpler —
   separate packets create a moment where the tree is inconsistent.
   Not a blocker.

3. **Linker gap.** Same as the rest of the arc. A type-name rename
   is very unlikely to regress behavior, and clippy + cargo check
   catch compile-level mistakes, so this is the least risky packet
   for skipped tests.

### Observation

Pure rename packets are low risk but high readability value. This is
the packet that finally stops the branch reading in two languages
(product name in SQL, feasibility name in Rust).
