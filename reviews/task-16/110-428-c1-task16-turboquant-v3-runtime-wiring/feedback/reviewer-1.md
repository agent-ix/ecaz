## Feedback: TurboQuant V3 runtime wiring — ACCEPTED, with caveats

Verified against:

- commit `851da10` (`Wire TurboQuant V3 hot/cold runtime paths`)
- diff stat: ~`1410` additions / `408` deletions across 9 files —
  build, graph, insert, scan, scan_debug, shared, vacuum, plus
  lib-level tests
- V3-format recognition via
  `GraphStorageDescriptor::TurboQuantHotCold(_)` and the
  hot+neighbor+rerank triple write pattern

### What's right

- **Every AM entry point actually picks up V3.** Build emits V3
  metadata and writes the hot/neighbor/rerank triplet; insert's
  empty-index bootstrap selects V3 for turboquant; vacuum
  understands hot tuples plus cold rerank payloads for pass 1 heap
  compaction, repair discovery, replacement-candidate collection,
  and finalize/deleted marking; scan's exact-score fallback and
  turboquant binary live-rerank logic accept both V1 and V3. This
  is the full-lifecycle port that packet `427` explicitly deferred.
- **Backward-compatible read path.** Scan / debug / page-decode
  helpers accept both legacy `TQ_ELEMENT_TAG` and V3
  `TQ_TURBO_HOT_TAG` tuples. That matches the reality that V1
  indexes on disk don't go away just because the writer moved to V3.
- **Rollover / tail-page tests refactored for the triplet shape.**
  Moving the dimension search to cheap shape math (rather than
  constructing a quantizer per candidate dim) is a small but real
  test-performance win and keeps the coverage meaningful.
- **Debug helpers understand both tags.** Packet `423`'s
  stage-profile helper still produces comparable numbers on V3, so
  cross-packet latency comparisons remain meaningful.

### Concerns

1. **Unclear whether `scripts/vacuum_concurrency_scratch.sh` was
   run against V3.** The 60-second concurrent INSERT + scan +
   VACUUM harness is the existing safety proof for vacuum changes
   (see review README lines 31–35). Unit-level coverage of passes
   1–3 is present, but this packet does not claim the concurrency
   scratch was re-run on a V3 index. **Please confirm or run it
   before the branch merges** — pass-level unit coverage does not
   prove cross-backend safety under real concurrent live insert.
2. **V3 is now the default writer on turboquant indexes, not an
   opt-in.** The packet does not call this out explicitly, but
   since build emits V3 metadata unconditionally for turboquant,
   new turboquant indexes on this head write V3 even without any
   reloption flip. That is almost certainly the right call (V1 is
   dormant on the write side), but it is a wire-format default
   change worth naming in the packet readout so merge reviewers
   aren't surprised.
3. **Insert/vacuum diff is large (~1000 lines net).** The code is
   straightforward (triplet writes, matching reads), but a second
   set of eyes on the layer-search / refill helpers that now
   operate on storage descriptors rather than inline scalar
   payloads would be worth it before merge. Specifically, the
   `cached_graph_element(...)` split in `scan.rs` that introduces
   `TurboQuantHotCold` as a distinct match arm.
4. **`INDEX_FORMAT_V1_SCALAR` is still recognized on read but not
   written.** Fine — but an existing V1 on-disk index re-opened on
   this head will read fine and then any new live insert writes a
   mix of V1 + V3 tuples on the same graph. Is that intentional
   (insert preserves existing format) or a bug (insert force-
   upgrades)? Worth an explicit contract line in the packet.

### Call

Accepted for the AM logic itself, **contingent on**:

- a yes/no on the `vacuum_concurrency_scratch.sh` run against V3
  (concern `1`)
- a clear statement on the existing-V1-plus-live-insert mixed-tag
  case (concern `4`)

Both are answerable without touching code. The packet itself is
well-scoped; these are merge-hygiene questions, not re-work.
