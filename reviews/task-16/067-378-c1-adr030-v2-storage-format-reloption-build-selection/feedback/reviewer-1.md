## Feedback: Storage Format Reloption Build Selection

Read `StorageFormat` in `src/am/options.rs:27-72`, the reloption parse at
`:264-284`, the `flush_build_state` switch at `src/am/build.rs:997-1004`,
and `BuildState::initial_metadata` at `src/am/build.rs:135-175`.

### What's right

- **Format choice now lives in index metadata, not process env.** This is
  the right architectural pivot for task 15 / ADR-032. Operators can now
  say `WITH (storage_format = 'pq_fastscan')` and have that intent
  persisted — no ambient process state required.
- **`StorageFormat::parse_reloption` errors on unknown values** instead
  of silently defaulting. The unit tests at `options.rs:357-372` prove
  both the accepted-values path and rejection of `legacy_format`. Good.
- **`initial_metadata` writes the grouped shape (with zero-length layout
  fields) for empty PqFastScan indexes** at `build.rs:154-174`. Pairs
  with `graph_storage_descriptor_accepts_empty_grouped_metadata` at
  `graph.rs:1478` so the metadata is decodable from creation onward.
  This makes the "format intent from creation" contract real, not
  deferred to first flush.
- **`TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` is gone from code and
  scripts.** Greps on the runtime tree confirm no `_BUILD` env
  references remain. That moves task 15's "No
  `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` references remain" definition
  of done forward.

### Concerns

1. **Default path for the grouped build still hardcodes
   `group_size=16`, `train_size<=1024`, `kmeans_iters=8`.** The packet
   acknowledges this but keeps them as module defaults instead of
   reloption-driven. Task 15 lists parameterization as a followup, and
   packet 391 partially addresses the group-size angle — worth naming
   explicitly that `train_size` and `kmeans_iters` are still frozen.

2. **The reloption parse calls `pgrx::error!` inside the option
   validation callback** at `options.rs:272`. That's consistent with
   pgrx idioms but worth a one-line confirmation that ALTER INDEX...
   SET (storage_format = ...) is either rejected or a no-op — switching
   a live index's reloption without REINDEX would be a footgun, and
   the README migration note says "switch format = REINDEX" but the
   option machinery doesn't enforce that.

3. **Same linker gap as the rest of the arc.** `cargo test` and
   `cargo pgrx test pg17` still fail at the PostgreSQL linker layer
   on this workstation. Reloption parsing has unit-test coverage,
   but the pg-facing side of the contract (grouped build with
   `storage_format='pq_fastscan'`, then ordered scan works) is only
   validated by `cargo check --tests` + clippy. That's a real gap for
   a slice that changes the user-visible CREATE INDEX surface.
   Whatever CI lane does run `cargo pgrx test pg17` needs to be
   confirmed green before merge.

### Observation

This is the most architecturally important packet in the 378–400 arc —
every follow-on (scan gate removal, insert parity, vacuum parity, docs)
depends on format choice living in reloptions rather than a process
env. Worth pinning that in the commit body so reviewers who only look
at late packets understand what enabled them.
