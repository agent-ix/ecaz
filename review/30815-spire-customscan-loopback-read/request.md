# Review Request: SPIRE CustomScan Loopback Read Fixture

Code slice for Step 3 of the ADR-067 CustomScan pivot. This adds the first
end-to-end PG18 fixture proving `EcSpireDistributedScan` can return a
remote-origin tuple through the ADR-068 tuple-payload path without the
superseded row-materialization catalog/register call.

## Scope

- Extends `ec_spire_remote_search_tuple_payload(...)` with `heap_block` and
  `heap_offset`, preserving the heap-coordinate fields that the production
  executor decode path expects while adding tuple payloads.
- Changes tuple-payload CTID resolution to preserve duplicate CTIDs by
  positional order instead of consuming a CTID-keyed map; this addresses the
  reviewer P2 from packet `30812`.
- Lets CustomScan request only simple projected relation columns from the
  target list, falling back to all non-dropped columns when projection analysis
  cannot narrow the request.
- Adds a PG18 loopback-remote fixture:
  - creates separate coordinator and loopback-remote tables/indexes,
  - rewrites coordinator leaf placements to a remote node,
  - registers the remote descriptor,
  - asserts `EXPLAIN` contains `Custom Scan (EcSpireDistributedScan)`,
  - runs `SELECT id, title ... ORDER BY embedding <#> ... LIMIT 1`, and
    verifies the returned row is the remote shard row.
- Updates the Phase 11 tracker with packet `30815` and keeps the final
  multi-instance distributed read lane open.

## Validation

- `cargo test customscan --lib`
  - 7 passed, including the new loopback-remote CustomScan read fixture.
- `cargo test tuple_payload --lib`
  - 6 passed, including duplicate missing-CTID preservation and the new
    loopback fixture.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.

## Review Focus

- Check that adding `heap_block` / `heap_offset` to the tuple-payload endpoint
  is the right compatibility shape for the production executor handoff.
- Check the positional tuple-payload resolution change for duplicate CTID
  behavior and empty-batch handling.
- Check the CustomScan projected-column selection. It intentionally handles
  only simple non-junk `Var` target entries and falls back to the full relation
  descriptor otherwise.
- Check the remaining boundary: this proves the read path through a loopback
  remote descriptor, not the final separate-instance distributed read lane.

## Artifacts

- `review/30815-spire-customscan-loopback-read/artifacts/manifest.md`
- `review/30815-spire-customscan-loopback-read/artifacts/cargo-test-customscan-lib.log`
- `review/30815-spire-customscan-loopback-read/artifacts/cargo-test-tuple-payload-lib.log`
- `review/30815-spire-customscan-loopback-read/artifacts/cargo-fmt-check.log`
- `review/30815-spire-customscan-loopback-read/artifacts/git-diff-check.log`
