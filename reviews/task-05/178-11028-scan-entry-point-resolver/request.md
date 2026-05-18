# Review Request: Scan Entry-Point Resolver (medoid-deleted fallback)

Branch: `adr034-diskann-access-method`
Author: coder-2
Target: `src/am/diskann/scan.rs`

## What this packet is

Pure-Rust isolated slice adding
`scan::resolve_entry_point(reader, preferred) -> Result<Option<ItemPointer>, String>`
which returns `preferred` when it's live, otherwise falls back to
`reader.first_live_tid()`. Closes the Phase 6B "medoid may be
deleted" open question from
`plan/design/diskann-scan-pgrx.md`.

Builds directly on packet 11027's `first_live_tid`.

## Why this

ADR-047 §10 defers live-medoid migration to rebuild. That means
the persisted metadata's `medoid_tid` may point to a tombstoned
element at scan time. The scan needs a deterministic, cheap
fallback so the `amgettuple` cursor never starts at a dead entry
point.

Semantic choice:

- Preferred TID is live → return it (no extra work).
- Preferred TID is missing, decode-corrupt, or tombstoned → fall
  back to the lowest-block live TID.
- Chain has no live tuples → `Ok(None)`. Phase 6B's scan path
  should treat this as "empty result set", not error.

Decode errors on the *preferred* TID are treated the same as
"not live" — the fallback will run and (unless the whole chain
is corrupt) succeed. This matches pgvectorscale's defensive
posture when a metadata page goes slightly stale versus the data
chain.

## Tests

Three new tests (SC-012..SC-014). Share a helper
`persisted_chain_with_tombstones(n, max_degree, to_tombstone)`
that persists a chain graph and flips `deleted` on the named
nodes via `vacuum::mark_deleted` + re-encode + `update_raw_tuple`.

- **SC-012** — live preferred TID returned as-is.
- **SC-013** — dead preferred TID falls back to `first_live_tid`
  and the result is distinct from the dead TID.
- **SC-014** — all-dead chain returns `None`; `INVALID` preferred
  also returns `None`.

## Verification

```
cargo check --lib     # clean
cargo test --lib am::diskann    # 84 passed (was 81)
```

## Non-changes (affirming choices)

- `ScanParams::entry_point` signature unchanged — callers keep
  passing a resolved `ItemPointer`. The resolver is a distinct
  primitive the Phase 6B `amrescan` callback wires in.
- No caching on the reader. Medoid resolution happens once per
  `amrescan`, not per `amgettuple`.
- `resolve_entry_point` does not log a warning when falling back.
  The fallback is a designed path (ADR-047 §10), not a surprise.
  Observability belongs in Phase 6B's EXPLAIN counters.

## Dependencies

- **Packet 11027** (reader iter/first_live_tid) — this consumes
  `first_live_tid`.
- **Packet 11022** (Phase 5D reader).
- **`plan/design/diskann-scan-pgrx.md`** — named this primitive
  under "Open questions — Fallback entry point."

## Not doing in this packet

- Migrating the medoid. ADR-047 §10 is explicit: live medoid
  migration is deferred to rebuild.
- Persisting the resolved fallback. Each `amrescan` re-resolves;
  if the rebuild scheduler bumps the medoid in the meantime, the
  next scan sees it.
- Multi-entry-point search (seed from k diverse live TIDs). Out of
  scope for V1; fold into a later Phase 6C pass if recall warrants.
