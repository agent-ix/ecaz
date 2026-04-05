# Review Request: Rescan Query Validation

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` no longer immediately hard-errors.
- It now rejects index quals, requires exactly one ORDER BY query, requires a non-NULL/non-empty `real[]` query, validates query dimensions against index metadata, and records minimal scalar state in the scan opaque.
- `amgettuple` still hard-errors, so no tuples are returned yet.

Review focus:
- ORDER BY query validation semantics
- Scan-key decoding safety and SQL-surface behavior
- Whether the current state recording is the right narrow boundary before tuple production exists

Questions to answer:
- Are the current `amrescan` preconditions too strict or still too loose for the existing SQL surface?
- Is decoding the query from `ScanKey.sk_argument` safe as written for this stage?
- Are there missing tests around NULL queries, multiple order-bys, or index quals?
