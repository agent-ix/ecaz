# 30307 SPIRE Retired Epoch Manifests — review

Code commit `e6b115ba`. Read `build.rs:551-565` (helper),
`insert.rs:161-167`, `vacuum.rs:389-395,500-506`, `mod.rs:710-789`
(snapshot dedupe), and the focused PG18 epoch-snapshot test.

## What landed

- `write_retired_epoch_manifest_to_relation`: clones the previous
  active manifest, flips `state` to `Retired`, zeroes
  `active_query_count`, validates, encodes, and appends. Reuses the
  same object-tuple page allocator as everything else.
- All three replacement publish sites (`publish_insert_delta_epoch`,
  `publish_compacted_delta_epoch_if_needed`,
  `publish_delete_delta_epoch`) call this helper after encoding the
  new manifest bundle but before writing it. Order is consistent
  across the three call sites.
- `index_epoch_snapshot` now dedupes manifests per-epoch by
  highest TID (block, offset). The deduped list feeds
  `plan_epoch_cleanup`; the original list still drives row emission,
  with non-latest tuples labelled `cleanup_blocked_reason =
  "superseded_manifest"` and `cleanup_eligible_now = false`.

## Architecture

The retired tuple is written before the new manifest bundle and
before root/control advances. Crash semantics:

- crash after retired write, before bundle write → original active
  manifest tuple is unchanged, root/control still points at it,
  active state is intact. The retired tuple is now an orphan
  duplicate for the previous epoch (no reader uses it; epoch
  snapshot dedupes it as the latest tuple but `cleanup_blocked_reason`
  becomes `active_root_manifest` because root/control still points
  there — wait, the dedupe picks the *retired* TID since it's
  newer. That means the active-epoch-still-pointing-at-old-TID case
  would mark the retired tuple as `superseded_manifest` only if the
  original was newer than retired... actually no — the dedupe just
  picks max TID, and the snapshot computes
  `is_active_root_manifest = root_control.epoch_manifest_tid == tid`.
  After this crash, root_control.epoch_manifest_tid still points at
  the *original* active tuple (not the retired one), so the original
  is `is_active_root_manifest = true` and
  `cleanup_blocked_reason = "active_root_manifest"`, while the
  retired tuple is `is_latest_manifest = true` with state=Retired
  and `cleanup_blocked_reason = "retention_window"` or similar. The
  original gets labelled `superseded_manifest` because it's not the
  latest TID. That contradicts root_control's view that it *is* the
  active root.
- crash after bundle write, before root/control advance → original
  active still authoritative; new bundle and retired tuple are both
  orphans; new bundle's active manifest is also a candidate for
  "latest tuple for epoch N+1" but root/control points at no such
  thing.
- clean publish → as designed.

The crash-recovery scenario above is worth a unit-level test: build a
synthetic snapshot with `(Active@tid_A, Retired@tid_B)` for the same
epoch where `tid_B > tid_A` *but* `root_control.epoch_manifest_tid =
tid_A`, and assert which row gets `is_active_root_manifest = true`,
which gets `superseded_manifest`, and that `cleanup_eligible_now`
stays false for both. The current PG18 test only covers the clean
publish case, so the dedupe interaction with crash residue is
unverified. Given this surface is being designed for incident
triage, the partial-write case is exactly the one operators will
hit.

## Correctness

- The dedupe key `(block_number, offset_number)` is fine for
  intra-page monotonicity but isn't strictly monotonic across pages
  if pages are extended out-of-order. In practice all three publish
  sites hold the index relation extension lock and write
  serially under one transaction, so the retired write always lands
  after the original active write, which means tid_retired > tid_original
  in lexicographic order. Worth a one-line comment in
  `write_retired_epoch_manifest_to_relation` documenting the
  ordering assumption it relies on for the dedupe to pick it.
- `cleanup_eligible_now = is_latest_manifest && cleanup_epochs.contains(...)`
  correctly prevents superseded duplicates from being marked
  reclaimable. Good defensive narrowing.
- `epoch_cleanup_blocked_reason` is only computed for the latest
  manifest; superseded rows shortcut to the literal
  `"superseded_manifest"`. This is the right structure — the cleanup
  planner shouldn't reason about non-latest tuples, and the label
  makes that explicit at the SQL surface.

## Style

- The retired-write call inside each publish path is duplicated
  three times verbatim. Could fold into a small helper
  `publish_replacement_epoch(...)` that takes the encoded manifests +
  previous active manifest, writes retired, writes bundle, returns
  locators. Worth doing while there are only three call sites and
  before split/merge in 30335 adds a fourth.
- `state: SpireEpochState::Retired, active_query_count: 0,
  ..previous_epoch_manifest` in `build.rs:557-560` — clear shape.
  No issue, but worth asserting in the helper that the input
  manifest's state is `Active` (not already Retired or some other
  value), so a future caller that passes the wrong manifest fails
  loudly rather than writing a degenerate retired-of-retired tuple.

## Status

Lands cleanly. The publication-order invariant (retired-before-bundle-
before-root-advance) is the right shape, and the snapshot dedupe is
sensibly defensive. Two follow-ups before this is treated as the
foundation for old-epoch reclamation:

1. Unit test for the partial-write residue case where
   `root_control.epoch_manifest_tid` points at the older TID for a
   given epoch.
2. Fold the three-site duplication into a single
   `publish_replacement_epoch` helper before split/merge (30335)
   adds a fourth call site.
