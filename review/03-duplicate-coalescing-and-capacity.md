# Review Request: Duplicate Coalescing And Inline TID Capacity

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- `aminsert` now coalesces duplicate encoded vectors into an existing element tuple rather than always appending a new element.
- Coalescing is limited by the current inline heap-TID capacity of the tuple format.
- When the tuple is already at capacity, insert rejects the duplicate instead of silently corrupting or overflowing state.

Review focus:
- Duplicate detection semantics
- Heap-TID append safety and bounds enforcement
- Interaction between duplicate coalescing and page reuse
- Whether failure mode/messages are coherent enough for current capability

Questions to answer:
- Is duplicate matching too weak or too strong for the stored encoding?
- Is the capacity boundary enforced exactly once and in the right place?
- Is there any path that can partially mutate a tuple before rejecting overflow?
