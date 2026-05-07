# 30361 SPIRE Phase 1 Landing — final review

Code commit `7fc2eb9c` ("Cover SPIRE scan sanity status labels");
landing packet adds packet-local unit logs and a manifest. This
review covers the cumulative state of `task30-spire-partition-
object-spec` against my prior round feedback (round-30303-30336)
and the Phase 1 scope as restated in 30359.

## Verdict

**Cleared to land.** The landing packet plus the work between
30337–30360 closes every concrete blocker I raised in the prior
round. The pq-fastscan deferral is the right scope move — clean,
bounded, diagnosed at the SQL surface, with a build-rejection
test (30327) and an empty-scan safety test (30329) fencing the
deferred behavior. No remaining concerns are landing-blockers.

## Round 1 → Round 2 closure check

Every recommendation from `round-30303-30336-feedback.md` traced:

| # | Concern | Resolved by | Verified |
|---|---|---|---|
| 1 | Diagnostics surface discoverability | 30345 + label-constants 30353/30356 | ✓ |
| 2 | Crash-recovery test for retired-manifest residue | 30342 / 30357 (bundle residue) + 30354 (precondition) | ✓ commit `2a30b8ce` `epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative` |
| 3 | Heterogeneous concurrency test | 30352 (insert × vacuum × scan) | ✓ |
| 4 | Three-site publish duplication | 30343 (`publish_replacement_epoch_to_relation`) | ✓ commit `8d8bc2e6` |
| 5 | Threshold formula extraction | 30348 | ✓ |
| 6 | `object_version` cross-check at compaction | 30344 (`require_compaction_leaf_object_version_match`) | ✓ commit `5281b248` |
| 7 | Checklist truth alignment | 30347 (scan path scope) + 30360 (validation checklist) | ✓ |
| 8 | 30308 leaf-snapshot manifest ordering | 30338 (`apply_leaf_snapshot_base_row` via `or_insert_with`) | ✓ commit `f22ecfb3` |
| 9 | EpochManifest magic prefix | 30351 (`EPOCH_MANIFEST_MAGIC = "ESME"`) | ✓ commit `a09bc0ca`, decode validates magic before format version |

Plus per-packet feedback closures:
- 30307 (retired manifests) refactor + crash-residue test → done
- 30308 ordering bug → fixed
- 30321 (cache refresh) seed-from-empty test gap → 30358 closes it
- 30335 (update mechanics doc) feedback → 30340 follow-up
- 30336 (concurrent insert) sleep-based race → 30341 (waiters
  poll-based barrier) replaces the 750ms sleep

Coder-1's response discipline is excellent. Every concern got a
named packet; the structural ones got code; the doc ones got doc
edits; the gaps got tests. No silent declines.

## What's actually landing

The Phase 1 scope as crystallized in 30359 + 30360:

- **Storage:** local single-store, single-level partition objects.
  Routing object → leaf objects, with insert/delete deltas folded
  into V2 segmented leaves at vacuum compaction.
- **Build/Insert/Vacuum:** all three publish paths share
  `publish_replacement_epoch_to_relation`. Retired-manifest trail
  written before bundle, bundle before root/control advance.
  Crash residue at every intermediate point is tested or
  diagnostically labeled.
- **Scan:** TurboQuant + RaBitQ scannable. Scan-descriptor
  root/control cache refreshes on epoch change (30339 strengthens
  30321). pq-fastscan populated builds reject at build time;
  empty pq-fastscan scans are safe.
- **Diagnostics:** 13+ `ec_spire_index_*_snapshot` SQL functions
  + an overview doc (30345) + named status labels (30353/30356)
  giving operators a stable contract.
- **Concurrency:** same-leaf concurrent insert (30336/30341) and
  insert × vacuum × scan heterogeneous (30352) covered.
- **Defensive:** epoch manifest magic prefix; compaction PID +
  object_version cross-checks; root + multi-root error paths in
  diagnostics; manifest dedupe across crash residue.

## What's intentionally deferred (and correctly so)

These are not blockers — the deferment is bounded and documented:

1. **pq-fastscan populated build/scan.** Diagnosed via
   `assignment_payload_status = 'deferred_model_metadata'`.
   Build rejects with a clear error; empty scan is safe.
   Re-enables when grouped-PQ model metadata + scorer binding
   land. Clean cut.
2. **Physical page reclamation / old-epoch cleanup.** Retired
   manifests are written and tracked; the `cleanup_eligible_now`
   diagnostic exists. Actual reclamation is a separate follow-up.
   The plan doc 30335 + retention semantics in 30340 define the
   contract; nothing in Phase 1 depends on reclamation firing.
3. **Remote placement, replicas, boundary-replica promotion.**
   Phase 0 architecture handled the spec; Phase 1 stayed local
   single-store. Replica deferral was checkpointed in 30325.
4. **Recall/latency measurement claims.** Diagnostic surface
   reports `recall_sanity_status` / `latency_risk_status`;
   measured benchmarks deferred to a separate gate. Right call —
   a Phase 1 landing review shouldn't conflate functional and
   performance acceptance.
5. **Split/merge execution.** Read-only thresholds + planning
   doc 30335 only. The publish helper from 30343 + the design
   doc give the next implementer a clear handoff.

## Residual observations (none blocking)

These are forward-looking notes for the next phase, not gates on
landing:

1. **Phase 1 lands with one publish lock as the only
   serialization point.** Insert, vacuum-cleanup, vacuum-
   compaction, and (eventually) split/merge all queue behind it.
   Today this is fine — workloads aren't pushing it. The first
   production deployment that runs a busy insert stream
   alongside vacuum will surface this as a contention story.
   Worth budgeting measurement before split/merge implementation,
   not after, so the lock-contention numbers are baseline-known.

2. **The diagnostic surface is the operator interface.** With
   13+ SQL functions, an overview doc, and stable label
   constants, operators have a real surface to reason about. But
   it's *all* there is — there's no `EXPLAIN`-side surface, no
   `pg_stat_*` integration, no log-line diagnostics. Fine for
   the foundation landing; worth tracking what operators
   actually reach for in the first weeks of use to prioritize
   next-round surface work.

3. **The retired-manifest contract is now load-bearing.** Three
   publish paths plus the dedupe walker plus cleanup planning
   all rely on the "retired-before-bundle-before-root-advance"
   ordering. The crash-residue test (30342) covers the most
   common partial-write case. Worth one more case in a future
   round: `Active@old_tid` *plus* `Retired@new_tid` for the same
   epoch, where `root_control.epoch_manifest_tid` got advanced
   but the bundle write was rolled back at PG transaction
   level. The current test covers the inverse (bundle written,
   root not advanced); the symmetric case isn't reachable today
   given the publish-lock discipline, but split/merge could
   change that.

4. **The "Phase 1 complete" text now matches reality.** 30347
   tightened the scan-path checklist, 30360 closed validation
   with named caveats. The checklist truth and the prose truth
   align — that's the bar I asked for. Keep this discipline as
   later phases land.

## Specific landing-packet notes

- **`artifacts/manifest.md`** is the right shape for a landing
  packet — names the unit logs, the validation, and the open
  measurement gate. Future landing packets in the SPIRE series
  should use this as the template.
- **The "current-sandbox rerun" note** about pgrx SQL paths is
  honest and the right call. Pointing reviewers at the prior
  30305 SQL validation for the scan-sanity row shape is fine;
  the new packet-local Rust unit tests lock the helper-level
  contracts that the SQL surface composes from.
- **No measurement claims** — explicit. Good. Don't let the next
  reviewer push for measurement here; that gate is rightly
  separate.

## Status

Land it. Phase 1 is structurally complete for the local
single-store / single-level / TurboQuant + RaBitQ scope. The
deferments (pq-fastscan, replicas, reclamation, measurement,
split/merge execution) are all bounded, named, diagnosed, and
have non-blocking handoffs to follow-up work.

This is a clean foundation. The discipline shown in the
30337–30360 response cycle — every concern named in a packet,
fixed in code or test or doc, traced through the validation
checklist — is the discipline you want carrying into Phase 2.
