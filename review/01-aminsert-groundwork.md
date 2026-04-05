# Review Request: Narrow `aminsert` Groundwork

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- `tqhnsw` now persists `seed` in metadata so live inserts can validate the build-time single-shape invariant.
- `aminsert` accepts a narrow append-only path:
  - decodes the incoming `tqvector`
  - validates `(dimensions, bits, seed)` against index metadata
  - appends one empty neighbor tuple plus one element tuple
  - initializes `entry_point` when inserting into a previously empty index

Review focus:
- Metadata invariant correctness for `(dimensions, bits, seed)`
- Empty-index initialization behavior
- Any WAL, locking, or page-state assumptions that are too weak for this narrow live path
- Whether callback-side errors stay coherent and SQL-visible

Questions to answer:
- Is there a concrete correctness bug in the current append-only `aminsert` path?
- Is there a missing regression test for a realistic edge case in this narrow scope?
- Is any metadata transition unsafe or under-validated?
