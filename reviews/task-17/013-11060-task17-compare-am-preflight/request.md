# Review Request: `compare pgvector` preflights the requested ecaz access method

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/compare/pgvector.rs`

## What this packet is

Packet 11058 fixed the missing-AM failure mode for `ecaz bench`, but
`ecaz compare pgvector --profile ec_diskann` still had the same blind
spot: it would happily run the ecaz-side KNN SQL even when
`<prefix>_corpus` had no `ec_diskann` index at all, then report a
"comparison" that was actually measuring a fallback path.

This packet applies the same preflight discipline to the compare
surface. If the selected ecaz profile's access method is absent on the
corpus table, the command now fails before it builds the pgvector
sidecar or runs any recall / latency work.

## What changed

### `crates/ecaz-cli/src/commands/compare/pgvector.rs`

Immediately after the existing corpus / queries table existence checks,
`run()` now verifies that `<prefix>_corpus` has at least one index using
`profile.access_method`:

```rust
if psql::index_count_with_am(&client, &corpus_table, profile.access_method).await? == 0 {
    return Err(eyre!(
        "{} on {:?}",
        crate::commands::bench::missing_am_error(profile, profile.access_method),
        corpus_table
    ));
}
```

Notes:

- It reuses the existing `psql::index_count_with_am()` helper from 11058.
- It also reuses the already-tested `bench::missing_am_error()` wording
  so the operator gets the same "`ecaz corpus load --profile X ...`"
  hint across both measurement surfaces.
- Placement is intentionally early: before `CREATE EXTENSION vector`,
  before sidecar materialization, before ground-truth compute.

That keeps the failure cheap and prevents `compare pgvector` from
creating extra state for a run that cannot produce a valid ecaz-vs-pgvector
comparison.

## Why this slice

- Same operator-integrity problem as 11058, just on the sibling compare
  path.
- Very small surface area: one early-return guard, no SQL-template or
  metric-code changes.
- Keeps the ecaz measurement tools consistent: both `bench` and
  `compare pgvector` now refuse to benchmark a profile that is not
  actually present on the corpus.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran local `pg18` verification outside the packet snippet:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- A dedicated pure helper in `compare/pgvector.rs` for the error copy.
  Reusing the bench formatter keeps the operator message aligned across
  surfaces and avoids a second string to maintain.
- Live integration coverage for the exact failure mode. The guard reuses
  helpers whose SQL and copy are already covered elsewhere; this packet
  keeps scope to the wiring.
- Any broader refactor of `bench` / `compare` into a shared measurement
  preflight module. One call site does not justify that indirection yet.
