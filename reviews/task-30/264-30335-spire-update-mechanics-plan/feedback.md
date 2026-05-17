# 30335 SPIRE Update Mechanics Plan — review

Doc commit `45ea0f25`. Read `plan/design/spire-update-mechanics.md`
in full and cross-referenced it against the existing
`spire-phase0-partition-object-storage.md` and the current
publication path in `insert.rs` / `vacuum.rs`.

## Strengths

- The split/merge "allocate new PIDs" rule (§Split, §Merge) is the
  right call. Reusing a PID across a coverage change would silently
  break the routing-graph invariant that "PID identifies a logical
  partition with stable centroid coverage" stated in the Phase 0
  note. Forcing new PIDs makes retired epochs trivially diffable
  against new ones — old PID disappears from the active routing
  object, new PIDs appear, retention handles the rest.
- Folding active deltas into the replacement leaf before publish
  (§Deltas and Visibility) reuses the vacuum-compaction visibility
  rules. Reusing one set of dedupe semantics for compaction *and*
  split/merge means the `vec_id`-based survival logic only has one
  authoritative implementation. Worth being explicit in the doc that
  this code is shared, not duplicated, when implementation lands.
- Concurrency story (§Concurrency): "use the existing publish lock"
  is the right starting point. It serializes split/merge against
  insert and vacuum cleanup at the same boundary the
  retired-manifest write (30307) already assumes, which avoids
  introducing a second lock-ordering discipline.

## Gaps and risks

1. **Routing object rewrite cost is unbounded.** §Split and §Merge
   both say "rewrite the parent routing object." For the
   single-level foundation, that's the root, which holds every
   centroid. A split that fires under load rewrites the entire root
   object. With a flat routing layout (30262) this is bytes-cheap
   but every concurrent scan reading the active root sees the
   replacement after epoch advance. Worth stating: split/merge cost
   on the routing side scales with `nlists`, not with the affected
   leaf, until hierarchy lands. Operators planning split rates need
   that.

2. **Replacement-PID allocation interacts with the PID allocator
   cursor.** The doc mentions allocating "replacement leaf PIDs"
   but doesn't say where they come from. They must come from the
   same `next_pid_to_allocate` cursor that insert uses, otherwise a
   concurrent insert publish that started before the split publish
   could allocate the same PID. The publish-lock invariant covers
   this in §Concurrency, but the allocator-cursor interaction
   should be called out as the *reason* the publish lock is needed
   for split/merge — not just for ordering.

3. **Rebalance §83 says "may reuse PID with a new object_version."**
   The §Rebalance contract assumes coverage is unchanged. But
   "coverage unchanged" is not directly observable from inputs — a
   centroid recomputed from the leaf's current rows usually drifts
   slightly. The doc needs a sharper definition: rebalance reuses
   PID only if the *centroid stored in the parent routing object*
   is unchanged byte-for-byte. Anything else is a split-of-1 or a
   one-leaf merge and must allocate a new PID. Without this, a
   "rebalance" that recomputes centroids slightly changes what
   queries get routed to that PID — silent recall regression on a
   PID that retained epochs are still returning hits from.

4. **Retention interaction with split/merge is unstated.** After a
   split publishes, the retired epoch holds the old PID. If a
   retained scan running against the prior epoch resolves a hit on
   the old PID, it must still be readable. Today this works because
   placement directories are per-epoch and old objects aren't
   reclaimed until retention expires. Worth stating explicitly: the
   split/merge plan does not change the retention contract; old
   PIDs remain readable for the retention window through their
   own placement entries. (This is implied by "old object tuples
   become cleanup candidates after retention," but the
   *queryability* of the old PID during retention should be
   asserted directly.)

5. **No backpressure / scheduling discussion.** The doc says
   "scheduler should treat those rows as advisory" but doesn't say
   *who* the scheduler is. Background worker? Vacuum hook? Manual
   `ec_spire_split(pid)` SQL call? Phase 1 is read-only triggers
   (30309), and that's fine, but the implementation packet that
   follows this design will have to pick. Worth a one-line "not
   yet decided; candidates are X, Y, Z" so the next packet has a
   choice surface.

6. **Cross-parent merge punt (§70-75).** The doc correctly defers
   cross-parent merge to "rebalance" once hierarchy exists. But
   "treated as a rebalance" is the wrong vocabulary if rebalance
   reuses PIDs and merge allocates new ones. Cross-parent merge by
   the Phase 1 definition still changes coverage, so it must
   allocate new PIDs *and* rewrite multiple parent routings. Worth
   renaming this case ("cross-parent merge becomes a multi-parent
   coverage rewrite") rather than overloading "rebalance."

## Status

Good Phase 2 anchor. Rule shape (immutability, new-PID-on-coverage-
change, delta folding, single publish lock) is consistent with the
Phase 0 model and with what 30307 already assumes about retired
manifests. Recommended edits before this is locked:

- Tighten §Rebalance with a byte-equal centroid criterion.
- Call out routing-object rewrite cost scaling with `nlists` for
  the single-level foundation.
- Note allocator-cursor interaction as the publish-lock
  justification.
- Affirm queryability of old PIDs during retention.
- Fix the cross-parent-merge vocabulary or scope it more carefully.
