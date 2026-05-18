## Feedback: PqFastScan Small-Dimension Group Size

Read `default_pq_fastscan_group_size` at `src/am/build.rs:1083-1086`
and `PQ_FASTSCAN_TARGET_GROUP_SIZE` at `:20`.

### What's right

- **Derives group size from the effective transform dimension.**
  `transform_dim.min(PQ_FASTSCAN_TARGET_GROUP_SIZE)` is the right
  shape: keep 16 where it fits, shrink when the transformed dim is
  smaller. No surprise behavior at typical workload dimensions.
- **The constant is renamed to `PQ_FASTSCAN_TARGET_GROUP_SIZE`,
  signaling "target, not mandate."** That's semantically correct
  now that small-dim builds can legitimately use a smaller group
  size.
- **Two layers of coverage.** A pure unit test in `build.rs` for
  the 8-dim plan shape, plus a pg test that builds the index and
  inspects persisted metadata. The unit test catches planner
  regressions; the pg test catches on-disk regressions.
- **Metadata is authoritative.** Because the runtime already
  drives everything off `search_subvector_count` /
  `search_subvector_dim` from metadata, the only code that needs
  to learn about the derived group size is the build planner.
  Runtime doesn't care.

### Concerns

1. **`min(transform_dim, 16)` means exactly one subvector for any
   transform dim ≤16.** That's a valid PQ4 shape but it's also a
   *trivial* one: the whole transformed vector is one group, and
   the codebook learns that single distribution. For small dims
   this is fine, but it's worth flagging that the "grouped"
   characterization is vestigial at that scale — the layout could
   degenerate toward plain single-codebook PQ. Not a bug, just a
   clarification for anyone reading recall numbers on small dims.

2. **Divisibility check stays strict.** If a future transform dim
   isn't divisible by 16, the build returns "transform dim N is
   not divisible by group_size M." The new derivation guarantees
   divisibility for dims ≤16 (group = dim) but leaves dims in
   (16, 31) at group=16 and potentially non-divisible. Worth
   confirming that `effective_transform_dim` always rounds up to
   a multiple that divides 16.

3. **Linker gap.** Same pattern as the rest of the arc; the pg
   test covers the new dimension-aware metadata shape end-to-end
   and did not run locally.

### Observation

Clean parameterization slice. Moves one hardcoded assumption off
the default path without changing default-workload behavior.
