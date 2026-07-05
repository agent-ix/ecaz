# Phase 2 deep review — packet 30389 through 30448 (60 commits)

This packet's feedback file consolidates the second-half-of-Phase-2 review.
Per-packet feedback exists for 30389 onward but the user-facing review is
written here in one document because the arc is best understood as a whole
composition.

## Triage of the 60 commits

Five sub-arcs:

| Arc | Packets | Risk | Depth |
|---|---|---|---|
| A. Validator hardening (closes prior-review feedback) | 30389–30407 | Low | Skim |
| B. Centroid + parent loader (merge side) | 30403–30409 | Medium | Read |
| C. Execution-input composition (relation/local × split/merge) | 30410–30433 | Low | Skim |
| D. **Atomic publish-lock + selected-plan entry seam** | 30417–30419, 30431–30433 | High | Deep read |
| E. **First user-visible SQL surface + shared publish lock** | 30439–30442 | High | Deep read |
| F. **Split materialization with k-means + heap-source bridge** | 30443–30448 | High | Deep read |

Arcs A and C continue the helper-per-commit cadence and pose no new
architectural risk; spot-reading code confirms the same pattern as the
previous arc (validate-shape → delegate to shared validator → carry
publish-plan-derived fields). Test posture: 374 unit tests passing,
up from 275 at last review.

## Arc A — validator hardening (closes prior feedback)

Packets that landed checks I or others flagged previously:
- `30401 Validate SPIRE scheduled execution parent contents` — fixes my
  cross-cutting concern that `replacement_parent.children()` was not
  re-checked against replacement-child PIDs in execution-input
  validation. Confirmed in `validate_scheduled_replacement_execution_publish_plan_parts`.
- `30402 Document SPIRE scheduler recheck selector contract` — adds the
  comment I asked for binding `recheck_leaf_replacement_schedule_decision`
  to the determinism of `choose_leaf_replacement_schedule`.
- `30407 SPIRE merge centroid shared duplicate validation` — folds
  duplicate-row guards from merge centroid into the shared scheduler
  rejection so both selector and centroid code see the same rejection.
- `30434` records the explicit feedback-response pass and adds the
  empty-centroid pass-through comment in merge/split leaf input
  validators.

Plus a wave of defense-in-depth validators (PID cursor bounds, leaf
object version ≠ 0, publish timestamp > 0, successor epoch checks, parent
object version, routing object version, snapshot-vs-decision active
epoch). Each one is its own packet and its own ~10-line check; none are
redundant — they catch different drift modes between caller and the
publish-lock plan.

## Arc B — merge centroid + parent loader

`build_scheduled_merge_replacement_centroids` recomputes the single merge
replacement centroid as a count-weighted average of the affected child
centroids. This is the right model for merge: coverage is the union, the
centroid is the union's centroid, and assignment counts are the natural
weights given that we don't reload source vectors during merge. Validates
merge decision shape, active epoch, parent PID, affected leaf row
coverage, duplicate affected rows, child centroid dimensions, affected-row
merge recommendations, and zero-count sparse merges.

`load_selected_scheduled_replacement_parent_routing` is the snapshot-side
loader that pulls the parent routing object from the active placement
directory by the decision's `replaced_parent_pid`. Validates that the
loaded object is actually a routing object (Root or Internal) of the
expected PID and contains every affected child PID.

`build_scheduled_merge_replacement_routing_parts` (1154-1174) composes
centroid + routing-child builder + parent rewrite into one merge
preparation seam.

## Arc C — execution-input composition

23 packets that compose what already exists into:
- `build_relation_selected_scheduled_{merge,split}_replacement_execution_input`
- `build_local_selected_scheduled_{merge,split}_replacement_execution_input`
- `..._from_snapshot` variants that load parent + folded leaf rows from
  the active snapshot rather than caller-provided
- `validate_selected_scheduled_replacement_*` preflights that enforce
  the bound (decision, lock_plan, input) tuple's self-consistency

Each composition is a thin wrapper around already-validated helpers. The
"selected" prefix denotes that the input is derived from a
`SpireSelectedScheduledReplacementPublishLockPlan` (decision + lock plan
together, atomic).

## Arc D — atomic publish-lock + selected-plan entry seam

`plan_scheduled_replacement_publish_lock` (2382-2401):
- Allocates PIDs on a **scratch copy** of the allocator
- Calls `plan_scheduled_replacement_publish_epoch` which validates
  every drift mode (root/control vs decision vs manifest epoch agreement,
  Published-state requirement, fresh PIDs, no duplicates, no PIDs below
  root_control.next_pid, every PID < pid_plan.next_pid, no cursor
  regression, successor epoch via checked_add)
- **Only commits the caller's allocator if both succeed**

`plan_rechecked_scheduled_replacement_publish_lock` (2403-2417):
- Calls `recheck_leaf_replacement_schedule_decision(rows, decision)` first
- Then `plan_scheduled_replacement_publish_lock`
- The recheck is against the *same `rows`* the decision was selected
  from. This is the lock-time freshness check: if the leaf snapshot
  collected under the publish lock disagrees with whatever the decision
  was originally derived from, fail before allocator advance.

`choose_scheduled_replacement_publish_lock_plan` (2419-2439):
- Selects decision via `choose_leaf_replacement_schedule(rows)`
- If `None`, returns `Ok(None)` — no allocator advance
- Otherwise builds the rechecked publish lock plan
- Returns `SpireSelectedScheduledReplacementPublishLockPlan { decision,
  lock_plan }` as one bound output

This three-layer composition is exactly what the design doc asked for:
"reload under the publish lock and re-check that the selected leaf PIDs
still satisfy the expected state" before allocator advance. The
`SpireSelectedScheduledReplacementPublishLockPlan` shape is what every
downstream "selected" execution input takes.

`publish_relation_selected_scheduled_replacement_epoch` (3006-3032) is
the live publisher: validates the (previous_manifest, snapshot, selected,
input) tuple via `validate_relation_selected_scheduled_replacement_publish_inputs`
then delegates to the existing `publish_relation_scheduled_replacement_epoch`.
The whole publish path now exists as a single function call from a
`SpireSelectedScheduledReplacementPublishLockPlan` plus the
relation-built execution input.

## Arc E — SQL surface + shared publish lock (first user-visible work)

Three real architectural events here:

**`30441 Share SPIRE publish relation lock`** moved insert and vacuum
publishing to a shared `lock_publish_relation` helper at
`src/am/ec_spire/mod.rs:64`. The lock mode is `ShareUpdateExclusiveLock`,
the guard is `Drop`-released. Insert delta publication, vacuum
delete/cleanup publication, and any future scheduler invocation now share
*one* lock contract. This is a small but structural prerequisite — the
live scheduler must serialize against insert/vacuum publish, and now
they all use the same lock mode against the same relation OID, so they
will mutually block correctly.

`SpireRelationLockGuard` lives in `mod.rs:53-72`. The guard takes a
`relid` rather than a `Relation` pointer for unlock so the unlock cannot
dereference a freed relation pointer; that's the right safety choice.

**`30439 SPIRE Maintenance Plan Snapshot`** adds the read-only SQL
function `ec_spire_index_maintenance_plan_snapshot(index_oid)` returning
`(active_epoch, planner_status, planned_action, planned_reason,
replaced_parent_pid, affected_leaf_pids, replacement_leaf_count,
replacement_leaf_pids, publish_epoch, next_pid, next_local_vec_seq,
planner_message)`. Calls `index_maintenance_plan_snapshot(index_relation)`
which:
1. Reads root/control
2. Returns `no_action / empty_index` if active_epoch == 0
3. Otherwise loads epoch manifests, builds validated snapshot, opens
   relation object store, collects leaf snapshot rows, and calls
   `maintenance_plan_snapshot_from_rows`
4. That helper invokes `choose_scheduled_replacement_publish_lock_plan`
   (Arc D's atomic entry) on a fresh `SpirePidAllocator::new(root_control.next_pid)`
   — a *scratch* allocator, so the read-only path never mutates
   anything

The result reports `planner_status = "planned" | "no_action"`,
`planned_action = "split" | "merge" | "none"`, the affected and
replacement PIDs, and the planned `publish_epoch / next_pid /
next_local_vec_seq` cursors.

**`30442 Locked Maintenance Plan Snapshot`** adds a sibling SQL function
that wraps the above with `lock_publish_relation(index_relation)` first.
This is the live preflight: it serializes against insert/vacuum publish,
loads the active state under the lock, derives the candidate, returns it,
*and releases the lock when the function returns*. So the lock is held
only for the planning duration.

This is a key surface, because:
- It validates that the publish-lock-held snapshot reads correctly
  against insert/vacuum concurrency
- It exposes the planner candidate in a way operators can inspect
  before triggering an actual replacement
- It is the closest precursor to the live scheduler entry — a future
  `ec_spire_maintain(index_oid)` would do the same lock acquisition then
  proceed past planning into execution

### Concern with the locked snapshot

`index_locked_maintenance_plan_snapshot` (mod.rs:1377-1382) takes the
publish lock, then calls `index_maintenance_plan_snapshot` which
internally opens manifests, opens the object store, collects rows, and
runs `choose_scheduled_replacement_publish_lock_plan` on a scratch
allocator derived from `root_control.next_pid`.

The snapshot returned to SQL therefore has a `next_pid` that reflects
"the cursor that *would* be advanced if execution were to publish",
**but the actual root_control on disk is unchanged** because the
allocator is scratch. That's correct for a read-only diagnostic, but the
returned `publish_epoch / next_pid / next_local_vec_seq` could mislead an
operator into thinking those values are committed. The
`planner_message = "scheduled replacement candidate selected for manual
publish"` partially mitigates this. Worth tightening the column docstring
or message to make explicit "cursor values shown are *projected* under
the next publish, not advanced". Minor.

### Concern with `lock_publish_relation` ordering

`lock_publish_relation` reads `(*index_relation).rd_id` *before* the
lock is acquired (mod.rs:67). For an already-`index_open`'d relation
this is fine (the relation is pinned), but if any future caller uses
`LockRelationOid` against a different OID (e.g., a fresh open that races
relfilenode swap), the unlock could target a different relid than what
was actually locked. Today there's no such caller — `index_relation` is
always already-open before `lock_publish_relation` is called — so this
is not a bug. Worth a one-line invariant comment on the helper:
"caller must hold an open `Relation` for `index_relation` for the
lifetime of the returned guard; we capture relid before locking and
unlock by relid in `Drop`." Minor.

## Arc F — split materialization with k-means

This is the largest piece of new algorithmic content in the whole arc.

**`build_split_replacement_leaf_materialization`** (833-928):
- Validates split decision, single affected PID, dimensions > 0,
  max_iterations > 0, source_rows non-empty
- Validates every source row: belongs to the affected base PID, is a
  visible primary assignment, is not a delta-insert row
- Calls `common_training::train_spherical_kmeans` with the source
  vectors (as `&[&[f32]]`), `dimensions`, `decision.replacement_leaf_count`
  as `k`, `seed`, and `max_iterations`. Reuses the *existing* k-means
  trainer used elsewhere in the codebase rather than rolling a new one.
- For each source row, calls `common_training::assign_vector_to_centroid`
  to get the centroid index, then routes the row's assignment into
  `routed_inputs[centroid_index]`.
- Calls `build_split_replacement_leaf_object_inputs(decision, pid_plan,
  routed_inputs)` which then validates that every replacement PID has at
  least one row, no duplicate vec_ids, etc.
- Returns `SpireSplitReplacementMaterialization { centroids, leaf_inputs }`
  ready to feed `build_scheduled_split_replacement_routing_parts`.

The two-pass shape (train then route) is right — k-means picks
centroids that minimize within-cluster spread, and the assignment pass
guarantees every source row lands in the correct cluster according to
the trained centroids. Reuse of `train_spherical_kmeans` and
`assign_vector_to_centroid` means the split trainer inherits the same
numerical behavior as the IVF build path's centroid training.

### Concerns with split materialization

1. **Empty-cluster handling**. `build_split_replacement_leaf_object_inputs`
   (already in the codebase) requires every replacement PID to have at
   least one input row. If k-means produces a centroid with zero
   assignments (which can happen when k > # distinct source rows or with
   pathological seeding), the materialization will fail with a
   `replacement routing child pid X has no leaf object input` error from
   the validator. The caller has no recovery. For a single-affected-leaf
   split with replacement_leaf_count = 2 and just 2-3 source vectors,
   this is plausible.

   Two options worth considering for the live scheduler:
   - Reseed and retry (deterministic if seed is bumped per attempt)
   - Reject the split candidate at scheduler-choice time when
     `effective_assignment_count < SPIRE_SPLIT_MIN_ROWS` (a new
     threshold); the existing thresholds gate sparsity but not
     "trainable size"

   Today the failure mode is a clean error rather than a corrupt
   publish, so this is not a correctness bug — it's a usability
   concern for the live scheduler.

2. **Seed and max_iterations are caller-supplied**. The eventual SQL
   entry will need a stable choice. Either an `ec_spire`-wide GUC
   (consistent with `seed` already used elsewhere) or a deterministic
   derivation from the decision (e.g., `seed = hash(active_epoch,
   replaced_parent_pid, affected_leaf_pid)`). Worth deciding before the
   SQL surface lands; otherwise the operator will have to pass these
   in by hand each call.

3. **`train_spherical_kmeans` choice**. The function name suggests
   spherical/cosine k-means rather than Euclidean. Confirm this matches
   the IVF/SPIRE distance contract — if SPIRE's primary distance is
   Euclidean and the centroids are stored as raw centroid vectors,
   training spherical-kmeans centroids may produce centroids whose
   norms don't match the source vector norms, which can subtly bias
   the parent routing (search distance to centroid). Quick check
   needed against the existing `ec_ivf` / SPIRE bulk-build training to
   confirm the same trainer is used there.

**`fetch_split_replacement_source_vectors`** (797-831): unsafe relation
helper that walks `replacement_rows`, calls
`load_indexed_source_vector_from_heap_row` for each assignment's
`heap_tid`, and collects `(heap_tid, source_vector)` pairs. Crucially,
it `continue`s when the loader returns `None` (which means the heap row
is dead/no-longer-visible under the snapshot). That looks fine for
*omitting* dead rows, but **the downstream
`build_split_replacement_source_rows` requires exact coverage** — every
assignment row needs a fetched source vector or the call fails with
`missing source vector for heap tid …:…`.

This is a **real correctness gap** for the live scheduler: if any heap
row referenced by an assignment is dead under the heap snapshot at
publish-lock time (e.g., a row that has been HOT-pruned but still has
an assignment referencing it pre-VACUUM), the split fails closed, but
that fail-closed *blocks all maintenance progress on this leaf* — the
scheduler will keep selecting the same candidate, keep failing on the
same dead row, and never publish. The vacuum path normally compacts
dead assignments, but vacuum itself competes for the publish lock with
maintenance, so a stuck split could starve vacuum's progress on this
relation.

Two ways to handle it:
- Drop dead-heap-tid assignments from the source-row set before
  calling materialization (treat them like `DELETE_DELTA` rows that the
  fold should already have caught — but if vacuum hasn't run yet, the
  fold won't have removed them)
- Have `fetch_split_replacement_source_vectors` and
  `build_split_replacement_source_rows` agree on a "skip dead rows but
  drop them from the assignment set used in materialization" contract

Worth resolving before the SQL entry lands. **This is the single
biggest open architectural question in this arc.** It does not block
the helper-level tests but will surface as a stuck-split bug the first
time live execution hits a heap that has uncompacted dead rows.

**`build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources`**
(1465-1505) is the unsafe wrapper that ties heap fetch to the existing
selected execution input builder. It's the function the eventual SQL
entry will call.

**Merge has no parallel `_from_heap_sources`** because merge's centroid
is recomputed analytically from existing parent centroids and active
leaf row counts — no heap re-read needed. Merge's
`build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot`
(1681) is the corresponding live entry. Asymmetry is correct.

## What's left to close Phase 2

1. **Live scheduler SQL entry** (e.g., `ec_spire_maintain(index_oid)`).
   Composition is one function:
   ```
   open heap relation + indexed attribute
   lock_publish_relation(index_relation)
   read root_control + active manifests + object_store
   collect leaf snapshot rows
   choose_scheduled_replacement_publish_lock_plan (scratch allocator)
   if None → return "no_action"
   if Split → build_..._split_..._from_heap_sources
   if Merge → build_..._merge_..._from_snapshot
   publish_relation_selected_scheduled_replacement_epoch
   advance the real allocator from selected.lock_plan.publish_plan.next_pid
   ```
   Probably 1-3 packets including a PG18 round-trip test.

2. **Heap-dead-row contract for split** (per Arc F concern above).

3. **Concurrency stress harness** — the only unchecked Phase 2 item.
   Should exercise insert + vacuum + split + merge under overlap; the
   shared publish lock means correctness is structurally guaranteed but
   the harness needs to prove it under load.

4. **Optional but high value**: split centroid seed/iterations
   policy, empty-cluster reseed strategy, and a documented "minimum
   trainable rows" threshold gating split candidacy.

## Status assessment

Phase 2 is at **~92% complete**. The composition is fully in place:
selecting a decision, taking the publish lock, atomically planning PID
allocation + publish epoch, loading source vectors from heap, training
split centroids, computing merge centroids, building rewritten parent
routing, writing replacement objects, and publishing the replacement
epoch — every step has a tested helper. The live entry is a thin
orchestrator that does not yet exist as a SQL surface, but the
`locked_maintenance_plan_snapshot` proves the lock acquisition and
candidate planning work end-to-end against a real relation.

The single biggest open architectural question is the heap-dead-row
contract for split source fetch (Arc F concern 3). Everything else is
either polish (Arc E concerns) or remaining scope (live entry +
concurrency stress + split centroid policy).

374 unit tests passing, helper composition is clean, prior-review
feedback is addressed. Coder is on track and producing very high-
quality code. Recommend they pause helper expansion long enough to
nail down the heap-dead-row split contract before wiring the SQL
entry.
