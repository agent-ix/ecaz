# Task 230 packet 001 reviewer seq-02 disposition

Reviewer feedback:
`../feedback/2026-08-28-02-reviewer.md`.

1. **Hot tombstone — removed.** The hot tuple and digest no longer contain a
   tombstone. Graph current/tombstone state remains the sole visibility gate;
   vec_id plus graph version/current validation covers misaddressing. Delete is
   graph-only and creates neither a page-sized hot rewrite nor locator churn.
2. **V2 length dispatch — resolved.** `tuple.rs:248-261` now appears in the
   contract with the required order: minimally bound version bytes, read and
   admit version, compute version-specific length, then compare. Existing
   `encoded_len` sites at `tuple.rs:184`, `tuple.rs:204`, and `insert.rs:867`
   take the version explicitly, alongside the previously named raw consumers.

No other accepted seq-03 contract is changed. Packet 002 remains gated on
outside acceptance of this seq-04 revision.
