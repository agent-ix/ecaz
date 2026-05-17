# Review Request: AM Build Graph And Page Staging Split

Scope:
- `src/am/mod.rs`
- `src/am/build.rs`

What changed:
- Moved build-time graph construction, entry-point selection, and staged data-page write helpers into `src/am/build.rs`.
- Kept the same build data structures and test coverage, but routed build-specific tests through the new module surface.
- Left `src/am/mod.rs` carrying scan execution and live insert behavior, with build internals now substantially reduced.

Review focus:
- Whether the moved graph/page staging helpers preserve current build behavior exactly
- Whether the new build-module surface is coherent enough to support one final cleanup pass on dead duplicate helpers
- Whether this split leaves `mod.rs` materially better positioned for later traversal work

Questions to answer:
- Do the extracted graph construction and page-staging helpers preserve current build semantics?
- Is the build module boundary now clean enough to remove any remaining dead build-only code from `mod.rs`?
- Does this leave the next serious work item correctly focused on scan traversal rather than more AM shell cleanup?
