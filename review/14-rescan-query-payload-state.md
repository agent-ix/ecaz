# Review Request: `amrescan` Query Payload State

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` no longer records only query dimensions in scan opaque state.
- It now copies the full `real[]` query payload into PostgreSQL-managed memory owned by the scan descriptor.
- `amendscan` frees that copied query state.
- Non-empty scan execution is still blocked; this slice only prepares state needed for later tuple production.

Review focus:
- Ownership and lifetime of the copied query payload
- Whether the `palloc`/`pfree` usage is correct for repeated rescans and scan teardown
- Whether the added regression coverage is enough for this narrow state-storage change

Questions to answer:
- Is storing the query payload in scan-owned PostgreSQL memory the right boundary for this stage?
- Is there any leak, double-free, or stale-pointer risk across repeated rescans and `amendscan`?
- Is there a missing regression test around repeated rescans with different payload values or scan teardown after rescan?
