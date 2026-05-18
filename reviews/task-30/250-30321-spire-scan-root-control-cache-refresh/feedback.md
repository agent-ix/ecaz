# 30321 SPIRE Scan Root-Control Cache Refresh — review

Code commit `576e8d11`. Read `scan.rs:286-345` and the new
`scan_opaque_refreshes_root_control_when_active_epoch_changes` unit
test.

## What landed

The previous scan-descriptor cache returned the first-seen
`SpireRootControlState` for the descriptor's lifetime. Now every
call to `cached_root_control_state_for_rescan` reads the
root/control page and compares `active_epoch` against the cached
copy:

- cache empty → use observed.
- cached epoch matches observed epoch → return cached (ignore other
  fields).
- cached epoch differs from observed → replace cache with observed
  and return it.

The split into `cached_*` and `observe_root_control_for_rescan`
makes the policy directly testable without a live relation.

## Correctness

- The cache no longer saves the buffer pin/lock — every rescan now
  reads root/control. That's a real behavior change. For nested-loop
  inner-side scans this is one extra page read per outer row. Not
  a correctness issue, but worth knowing the cost was traded for
  freshness; a comment near the cached-getter would document
  intent.
- "Same epoch returns cached" means cursor-shaped fields
  (`next_pid_to_allocate`, `next_local_vec_seq`,
  `epoch_manifest_tid`, etc.) can be stale relative to the observed
  state when the epoch hasn't advanced. The unit test exercises this
  exact case (`same_epoch_newer_cursors` returns the *cached*
  `epoch_one`, not the newer cursors). Today the scan path doesn't
  read those allocator cursors, so this is safe — but it's a
  fragile invariant. If a future reader of `root_control` on the
  scan side ever queries a cursor field, it will silently see stale
  data within a single epoch.

  Two reasonable fixes:
  - Always replace the cached copy with `observed`, dropping the
    "cached if epoch matches" branch. The page read already
    happened; the comparison saves only a struct copy.
  - Or: narrow the cache to a `SpireScanRootControlView` struct
    holding only the fields scan actually consumes (active_epoch,
    epoch_manifest_tid maybe), so cursor staleness can't be
    misused.

  The first is simpler; the second is more defensive. Either is
  better than the current "trust me, scan only reads epoch" comment.
- The published-vs-empty distinction: `read_root_control_page`
  always returns a `SpireRootControlState`. If the relation has
  never been built (empty pre-publish), what does it return? The
  test uses `SpireRootControlState::published(...)` only. If
  `read_root_control_page` panics or returns `Empty` in the
  pre-publish case, this code path is fine because rescan only
  fires after build, but would be worth one assertion.

## Test coverage

The unit test is well-shaped: three observations across two epochs,
cursor change within an epoch is the middle case. Good coverage of
the policy. Gap: no test that exercises the "cache empty → seed"
branch. Today the first call falls into `if let Some(cached)` =
None and goes straight to `self.root_control = Some(observed)`. A
two-line addition (assert pre-call `opaque.root_control` is `None`,
then call and assert it equals observed) would lock the seed-from-
empty behavior alongside the cross-epoch transition.

## Status

Solid hardening. The functional change is correct and the test
documents the intent. Recommend dropping the "same epoch returns
cached" optimization — the page read already happened, so the
optimization saves only a struct copy at the cost of cursor
staleness as a hidden footgun. If you keep it, narrow what's cached
to fields scan is committed to using.
