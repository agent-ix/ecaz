# Review Request: AM Build Tuple And Source Scan Split

Scope:
- `src/am/mod.rs`
- `src/am/build.rs`

What changed:
- Moved build tuple decoding from tqvector datums into `src/am/build.rs`.
- Moved `build_source_column` heap scan plumbing and slot/datum helpers into `src/am/build.rs`.
- Left graph construction, entry-point selection, and staged page writes in `src/am/mod.rs` so the slice remains mechanical.

Review focus:
- Whether the build tuple decoding and source-scan behavior are preserved exactly after extraction
- Whether the helper visibility opened between `mod.rs` and `build.rs` remains appropriately narrow
- Whether this is still a clean intermediate step before moving graph/page build helpers

Questions to answer:
- Do the moved tuple decode and source scan helpers preserve current build behavior exactly?
- Are the remaining shared build types and helpers exposed narrowly enough across the module boundary?
- Is the next logical split now the graph/page staging helpers rather than more scan or insert work?
