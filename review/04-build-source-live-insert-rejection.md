# Review Request: Rejecting Live Inserts For `build_source_column`

Scope:
- `src/am/mod.rs`
- `src/lib.rs`
- `sql/bootstrap.sql`

What changed:
- Index builds may still use a configured raw `real[]` source column for graph construction.
- Live `aminsert` now rejects such indexes instead of trying to mix raw-source build semantics with code-byte-only insert semantics.

Review focus:
- Whether this restriction is the right current boundary
- Error-path clarity and SQL-surface behavior
- Any overlooked cases where the AM could still accept an unsupported live insert for a source-column index

Questions to answer:
- Is there any hole in the current rejection path?
- Is there a smaller or clearer SQL-surface restriction that would better reflect current capability?
- Are there missing tests around reloptions or index definitions here?
