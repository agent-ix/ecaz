# Review Request: Persist Gamma In Element Tuples

Scope:
- `src/am/page.rs`
- `src/am/graph.rs`
- `src/am/build.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `spec/adr/ADR-013-persist-gamma-in-element-tuples.md`

What changed:
- Added `gamma: f32` to the persisted `TqElementTuple` payload and updated element-tuple encode/decode/size accounting accordingly.
- Plumbed stored `gamma` through the shared graph read surface via `GraphElement`.
- Updated both build-time staging and live `aminsert` element writes to persist gamma into the index.
- Added regression coverage that build and insert paths retain distinct persisted gamma values for same-code tuples.
- Accepted ADR-013 to document the page-layout change and the next follow-on slice to remove heap-fetch scoring from scan and duplicate-detection hot paths.

Review focus:
- Whether the new element-tuple layout is coherent and low-risk for the current pre-v1 format stage
- Whether persisting gamma at build and live-insert time covers all current element writers
- Whether the first checkpoint stops at the right structural boundary before switching scan scoring and duplicate checks over to the persisted value

Questions to answer:
- Is the chosen gamma field position in the element payload reasonable for this format revision?
- Are there any remaining element tuple construction or decode sites that should have moved in the same checkpoint?
- Is the follow-on boundary clear enough: next remove heap-fetch gamma reads from scan scoring and duplicate detection?
