## Feedback: Persisted `rerank_source_column` reloption — ACCEPTED

Verified against:

- commit `a3e1ea0` (`Persist pq_fastscan rerank_source_column
  reloption`)
- `src/am/options.rs`: new `rerank_source_column` reloption,
  storage-format guard (`pq_fastscan` only), wired through
  `TqHnswOptions`
- `src/am/scan.rs`: precedence `env_override >
  rerank_source_column > build_source_column`;
  `PqFastScanRerankModeResolution::
  DefaultHeapF32WithRerankSourceColumn` resolution variant;
  grouped-heap-rerank error text now names all three selectors
- new unit test
  `rerank_source_backed_pq_fastscan_default_rerank_resolves_to_
  heap_f32` pinning default flip, resolution name, and source
  precedence on one options row

### What's right

- **Scope is `pq_fastscan` only.** The storage-format guard in
  `relation_options(...)` rejects `rerank_source_column` on
  non-`pq_fastscan` indexes at option-parse time, not later. That
  is the right surface for an error — a user who sets the
  reloption with the wrong storage format finds out at `CREATE
  INDEX`, not at query time.
- **Build-time validation matches.** Empty and non-empty build
  both validate against the heap relation using the shared
  source-column resolver with the `real[] or bytea` type policy,
  and resolver error messages now name the caller label. So a
  wrong column type fails loudly at build with clear attribution.
- **Precedence order is right for debug/measurement.** Env
  override staying on top of persisted reloption means measurement
  packets like `432` can still force a different source without
  rebuilding. That is the correct operational posture even if it
  complicates the "is the persisted default actually in effect"
  readout (see concern `2`).
- **`build_source_column` semantics preserved.** Keeping
  `build_source_column` for grouped derivation and introducing
  `rerank_source_column` as the narrower "raw rerank input"
  control is the right separation. It does not force users onto a
  single column for both roles.
- **Coverage breadth.** pg fixtures can now construct a
  `pq_fastscan` index with persisted `source_raw`, and the new
  tests pin: runtime-settings reports the persisted source; exact
  heap scores without env override; non-pq_fastscan rejection;
  missing-column failure; wrong-type failure; default mode flip.
  All five of these needed coverage to lock in the new option.

### Concerns

1. **`rerank_source_column`'s `pq_fastscan` restriction leaves
   turboquant users stuck on `build_source_column`.** On the
   serious lane today turboquant builds write V3 hot/cold (packet
   `428`), and the measurements in `429`/`430` used the
   pq_fastscan rebuild shape to reach `source_raw`. That is
   consistent with the readout but means the "productized" win
   is *not* available to users running V3 turboquant indexes. Is
   that intentional (turboquant scans don't currently consume
   `rerank_source_column` because rerank is policy-quantized by
   default) or a follow-on gap? Worth naming explicitly.
2. **Restart-helper bug is a packaging hazard.** Packet `432`'s
   §4 flags that `scripts/restart_adr030_scratch.sh` always
   exports `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`, which means a user
   (or a future reviewer) following the established helper will
   not actually measure the persisted default this packet claims
   to ship. This is worth fixing alongside the reloption — either
   drop the forced env export when the requested mode matches the
   Rust default, or teach the helper about the persisted
   reloption. Low code cost, high surprise-reduction.
3. **Stale-TID constraint is still open.** Packet `430`
   demonstrated that `ALTER TABLE ... ADD COLUMN` + backfill on
   an already-indexed table requires `REINDEX`. This packet's
   §4 acknowledges it but defers. Before closing task 16, pick
   one of: (a) explicit doc + error-message work, (b) a tracked
   follow-on task with owner + date, or (c) accept as a known
   constraint and say so in the task readout.
4. **Validation path uses "shared source-column resolver."**
   Good reuse, but this means any future change to that resolver
   silently changes `rerank_source_column` semantics. Worth a
   locked test that explicitly asserts the accepted types are
   `real[]` or `bytea` (not e.g. `float4[]`) — I could not find
   one pinning the negative cases by type.

### Call

Accepted as a productization of packet `430`'s measurement. Ship
alongside the restart-helper fix and a clear statement on the
turboquant-turnoff question.
