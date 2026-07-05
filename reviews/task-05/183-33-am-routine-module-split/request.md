# Review Request: AM Routine Module Split

Scope:
- `src/am/mod.rs`
- `src/am/routine.rs`

What changed:
- Extracted AM routine assembly, `amvalidate`, and the SQL handler entrypoints from `src/am/mod.rs` into `src/am/routine.rs`.
- Kept the existing callback wiring unchanged, with the new module referencing the same build, insert, scan, cost, vacuum, and options callbacks.
- Left behavior unchanged so this remains structural preparation before scan traversal work.

Review focus:
- Whether the extracted routine module preserves the exact callback registration and SQL entrypoint behavior
- Whether the dependency boundary between the new routine module and the remaining callback implementations is still clear
- Whether this split improves the review surface without obscuring AM registration flow

Questions to answer:
- Does `src/am/routine.rs` preserve the same `IndexAmRoutine` callback table as the prior inline implementation?
- Are the handler entrypoints still exposed at the right symbol names and visibility after extraction?
- Is this a clean stopping point before extracting larger build-side code from `src/am/mod.rs`?
