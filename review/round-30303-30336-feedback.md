# SPIRE Round Review: 30303–30336

Reviewing 34 packets landed since 30302 (last per-packet feedback,
commit `c8edc1ea`). Branch `task30-spire-partition-object-spec`,
range commits `fef3ec03` through `1ef9dae8`. Per-packet feedback
exists for the architecturally significant items:

- `30307-spire-retired-epoch-manifests/feedback.md`
- `30308-spire-leaf-partition-diagnostics/feedback.md`
- `30321-spire-scan-root-control-cache-refresh/feedback.md`
- `30335-spire-update-mechanics-plan/feedback.md`
- `30336-spire-concurrent-insert-coverage/feedback.md`

This document covers the rest as a coherent round: the diagnostic
SQL surface (12 packets), test/coverage additions (9 packets), the
30320 vacuum hardening, and the seven milestone rollups.

> **Round 2 addendum.** Initial round summary was written from a
> sampling — diagnostic-cluster code (30303–30315) had been
> categorized via packet `request.md` only. A second pass through
> the actual code surfaced one real correctness bug (30308 manifest-
> iteration order, see per-packet feedback) which propagates into
> 30309 and 30310 since they consume the same row collection. Two
> recommendations from the original list have been adjusted below:
> the allocator-exhaustion suggestion was already implemented in
> 30315 and is struck; an EpochManifest tuple-collision concern is
> added.

## Architecture & data-model summary

The round consolidates Phase 1 to a "complete first-draft" state.
The shape that's emerged is solid:

- **Object immutability + retired-manifest trail** (30307) is the
  right foundation. Every replacement publish now leaves a
  retention-eligible breadcrumb in the same relation as live
  manifests. This is what makes split/merge (30335) implementable
  without redesigning publish.
- **Per-epoch placement directory** with `pid → local_store_id →
  object` is doing real work in the diagnostic surfaces. The
  consistency between `placement_object_bytes` (30299) and
  `child_object_bytes` (30302) and `root_routing` is held together
  by a single placement entry shape — operators reading any
  diagnostic see the same byte numbers for the same PID. This is a
  good invariant; preserve it as new diagnostics land.
- **Logical `vec_id` as dedupe identity** is being upheld through
  delete-delta semantics, snapshot scan filtering, and (per 30335)
  split/merge folding. As long as this stays the only dedupe
  identity, retention-window queryability of old epochs falls out
  naturally.
- **Single publish lock** is the serialization point for insert,
  vacuum-cleanup, vacuum-compaction, and (per design) split/merge.
  This is the discipline holding the round together. It's not
  contentious yet because there are no batched writers, but the
  moment a batching path lands (insert batching debt, 30310, is
  the first hint) it will become the bottleneck. Worth budgeting
  for measurement before that path closes.

What's **not yet defended** at the round level:

- No end-to-end crash-recovery test exercises the partial-publish
  states the design relies on (retired-written/bundle-not-written;
  bundle-written/root-not-advanced). 30307 assumes these cases are
  benign; nothing yet proves it.
- No retention-window expiry test. The "old PID is still queryable
  during retention" invariant 30335 leans on is implicitly true
  but unverified.
- Concurrency coverage (30336) is single-shape (two identical
  inserts). Heterogeneous (insert × vacuum, insert × scan-during-
  publish) is the workload that would exercise the lock
  ordering most usefully, and is also the workload split/merge
  will inherit.

## Diagnostic SQL surface (30303–30315, minus 30307)

**Packets:** 30303 PQ-FastScan deferral, 30304 relation storage debt,
30305 scan sanity, 30306 epoch cleanup, 30308 leaf partition,
30309 leaf maintenance thresholds, 30310 insert batching debt,
30311 hierarchy, 30312 partition object, 30313 delta, 30315
allocator.

This cluster reads as one coherent surface designed in 11 commits.
The pattern is uniform:

- pure Rust function in `mod.rs` returning a `Vec<...SnapshotRow>`,
- pgrx wrapper in `lib.rs` that opens the index under
  `AccessShareLock`, calls the helper, closes,
- focused PG18 test asserting both empty-index and populated-index
  shapes.

What's working:

- The "label-not-enum" pattern (`placement_state_name`,
  `partition_object_kind_name`, `assignment_payload_status`, etc.)
  keeps the SQL surface stable against Rust enum reorderings, and
  keeps every label `&'static str` so per-row formatting is
  branch-free.
- Empty-index short-circuits (`active_epoch == 0` in 30302; same
  shape repeated in 30303–30315) avoid opening the relation object
  store before there's anything to read. Cheap and correct.
- Aggregate cross-checks in tests (`bool_and(parent_pid =
  root_pid)`, `sum(child_assignment_count) = inserted_rows`) verify
  internal consistency rather than mere existence. This idiom
  should remain the test-quality bar for new diagnostics.

Concerns:

1. **Surface area is growing fast.** 12 SQL functions in this
   round, each returning 6–18 columns. There is no consolidated
   list anywhere I could find of which `ec_spire_index_*_snapshot`
   functions exist, what they're for, and which is the
   recommended starting point for a given operator question. By
   the time hierarchy adds another half-dozen, an operator will
   have to read source to know which to call. A single doc page or
   even a top-of-file comment listing the functions, their
   audience (operator-facing vs. debugging-only), and a
   "start-here" recommendation would pay off heavily.

2. **Scannability semantics in 30303 are fragile.**
   `assignment_payload_status = 'deferred_model_metadata'` for
   `pq_fastscan` is a useful operator hint, but the *reason* —
   that grouped-PQ model metadata is not yet persisted — is
   encoded only in the recommendation string. Once that metadata
   lands, the status string changes and any operator
   tooling/dashboard keying off the literal will silently
   misbehave. Worth: (a) listing the exhaustive set of valid
   status strings somewhere stable (a Rust enum already wrapped by
   `assignment_payload_status_name`?), and (b) committing to
   never reuse a string for a new meaning.

3. **`scan_object_tuples` (30304) holds buffer locks during
   visit.** The visitor closure is called under
   `BUFFER_LOCK_SHARE`. This is fine for the current
   accumulator-into-Vec usage, but it's a sharp tool. A future
   diagnostic that does anything inside the closure that pins or
   reads another buffer (even transitively, via `ec_spire`
   helpers) risks a buffer-pin deadlock. Worth a doc comment on
   `scan_object_tuples` documenting the contract: visitors must
   not perform I/O that touches another page in the same
   relation. Better still, restructure to copy tuple bytes out
   under the lock and call `visit` after release.

4. **30309 thresholds are thresholds-only.** The split/merge
   recommendations are computed from current state. Good. But the
   formula `max(32, 4 * ceil(total / count))` is hardcoded in two
   places (30335 plan + 30309 code). Pull them into one constant
   so a future tuning change doesn't drift between the
   recommendation surface and the design doc.

5. ~~**Allocator diagnostics (30315) report cursors but not
   exhaustion thresholds.**~~ **Struck after re-reading the code.**
   30315 already exposes `remaining_pid_allocations`,
   `pid_near_exhaustion`, `remaining_local_vec_id_allocations`,
   and `local_vec_id_near_exhaustion`, parameterized by a
   `warn_within` argument. This recommendation was based on the
   request.md summary, not the code. Apologies — the surface is
   already operator-ready.

6. **`EpochManifest` tuple-stream filter is fragile (30306, 30307
   dedupe).** `index_epoch_snapshot` discovers manifest tuples by
   walking *all* object tuples in the relation and accepting any
   tuple whose length equals `EPOCH_MANIFEST_BYTES` (36) and whose
   first two bytes parse as `META_FORMAT_VERSION`. The decode
   then validates state ∈ {1..4} and consistency ∈ {1,2}. There's
   no magic-byte sequence specific to epoch manifests — the same
   `META_FORMAT_VERSION` prefix is shared with placement entries,
   manifest-bundle entries, and other meta encodings.

   Today this works because no other meta encoding is exactly 36
   bytes (placement_entry is 50, manifest_entry is 34,
   placement_directory_header is 20). But that gap-from-collision
   is incidental. The first time a new meta type with a 36-byte
   encoding is added, or a leaf assignment row with a specific
   vec_id length collides, the dedupe path silently picks up
   garbage rows.

   Cheap fix: prepend a 4-byte `EPOCH_MANIFEST_MAGIC` to the
   encoded manifest (parallel to `ROOT_CONTROL_MAGIC`), bump
   format version, and check it first in `decode`. The dedupe
   walker becomes structurally robust against new tuple shapes.

7. **30308 manifest-iteration ordering bug.** The Leaf branch's
   unconditional `insert` overwrites accumulated Delta counts when
   manifest order is Delta-before-Leaf for the same parent PID.
   Latent today (no foundation path produces that order), but the
   diagnostic asserts a contract over arbitrary order. See
   per-packet feedback.

## Test/coverage packets (30314, 30316–30319, 30322–30324, 30327, 30329)

These are mostly fine and mostly cheap. Two notes:

- **30316 (root-routing defensive coverage)** is exactly the
  synthetic-snapshot test the 30302 feedback asked for — the
  multiple-root and no-root error paths now have unit coverage.
  Good follow-through.
- **30319 (multi-row insert epoch coverage)** asserts that a
  five-row `INSERT ... VALUES ...` advances the active epoch by
  five (one per row). That's the current insert-delta-per-row
  contract, and the test locks it in — but the contract itself is
  what insert batching (the open Phase 1 `Insert batching debt`
  item) will eventually break. When that lands this test will
  need to flip its assertion. Worth a comment in the test pointing
  at the eventual batching gate so the failure message is
  self-explanatory when batching arrives.
- **30324 (NULL insert error path)** is a one-line correctness
  test — exactly the right size.
- **30327 / 30329** correctly bracket PQ-FastScan's deferred state:
  populated build is rejected, empty scan is safe. Together these
  fence the deferral until the grouped-PQ metadata work is real.

## 30320 vacuum compaction leaf PID guard

Renaming the lookup key from `header.pid` to `manifest_entry.pid`
and validating they match before rewriting is the correct hardening.
Catches a manifest/header divergence — e.g., a bug where the
manifest entry got copied to the wrong PID — before a rewrite
silently associates rows with the wrong base leaf. Two unit tests
cover the pid-match check itself. Solid; no action.

The only thing missing: a parallel guard for `object_version`. The
manifest entry carries a version; the header carries a version;
neither is currently cross-checked at compaction time. Same class
of bug, same one-line `require_compaction_leaf_object_version_match`
helper. Consider adding alongside.

## Milestone rollups (30326, 30328, 30330–30334)

Seven plan-only commits flipping `[ ]` to `[x]` on Phase 1
checklist items. They're book-keeping and that's fine. What I'd
push back on:

- **30331 "Diagnostics complete" is over-claimed.** The diff
  changes the language to "deeper operator guidance remains open
  under the review/measurement gate rather than the Phase 1 admin
  diagnostic surface." That's a reasonable scope move, but
  combined with the 12 new diagnostic functions and no
  consolidated operator-facing doc, "complete" reads optimistic.
  At minimum: the next packet under this scope should be a
  `ec_spire_diagnostics_overview` SQL function or a doc page that
  enumerates the functions and their audience, before this is
  treated as a closed item.
- **30330 "Scan path complete"** with the open caveat
  "TurboQuant/RaBitQ only" — fine, but the open-caveat list lives
  in the prose paragraph of the task doc, not in the checklist.
  An operator scanning the checklist sees `[x]` and infers full
  coverage. Either invert (leave `[ ]` until PQ-FastScan is in)
  or split this into "Scan path (TurboQuant/RaBitQ) [x]" and "Scan
  path (PQ-FastScan) [ ]" so the checklist truth is the prose
  truth.
- **30326, 30328, 30332, 30333, 30334** are honestly scoped — each
  is "Phase 1 X complete with these caveats" and the caveats are
  named in the prose. No issue.

## Cross-cutting recommendations

1. **Add a doc page enumerating `ec_spire_index_*_snapshot`
   functions.** Single source of truth for operators. Counter the
   diagnostic surface sprawl before hierarchy adds more.
2. **Crash-recovery test for retired-manifest residue.** Per
   30307 feedback. Most important durability gap in the round.
3. **Heterogeneous concurrency test** (insert × vacuum × scan).
   Per 30336 feedback. Most important concurrency gap.
4. **Refactor the three publish call sites into a single
   `publish_replacement_epoch` helper** before split/merge adds a
   fourth caller. Per 30307 feedback.
5. **Extract the 30309 split/merge threshold formula to a
   constant** referenced from both the code and the design doc.
6. **`object_version` cross-check at compaction time** — the
   missing parallel to 30320's pid guard.
7. **Tighten 30330/30331 checklist semantics** — keep the
   checklist truth aligned with the prose caveat list.
8. **Fix 30308 leaf-snapshot manifest ordering.** Two-pass row
   construction, leaves first then deltas, eliminates the latent
   overwrite. Same fix de-risks 30309 and 30310 which consume the
   row collection.
9. **Add an `EpochManifest` magic prefix.** Today the dedupe scan
   filter is "tuple length 36 + valid format-version byte". Adding
   a 4-byte magic at encode time and checking it first in decode
   makes the scan structurally robust against future tuple-shape
   collisions (parallel to `ROOT_CONTROL_MAGIC`).

## Status

This is a strong round. The architectural through-line — immutable
objects, retained-then-retired-then-cleaned manifests, single
publish lock, vec_id-as-dedupe-identity — held up across 34 small
packets. Per-packet quality is high; tests assert structural
invariants rather than just existence.

The risk profile is shifting. With Phase 1 nominally closed, the
next gates are durability under crash and behavior under
heterogeneous concurrency. Both are presently asserted by design
but not by test. Those two gaps, plus the diagnostic-surface
discoverability problem, are what I'd want closed before
split/merge implementation starts and turns the publish lock into a
contended resource.
