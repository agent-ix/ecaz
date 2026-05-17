# Task 31 IVF Score-Ranked Probe Order

Reviewer: please review this Task 31 implementation checkpoint.

## Scope

This slice changes `ec_ivf` probe execution order after centroid selection.

Before this patch, `build_selected_probe_plan` sorted selected probe lists by
list id / physical block order before building the posting block sequence. That
discarded the centroid score ranking from `select_probe_lists`, so lower-value
lists could be scanned before the best lists had tightened the PQ-FastScan
bound.

After this patch:

- selected probe lists preserve the original centroid-ranked order
- overlapping posting ranges are deduplicated in first-range order instead of
  being globally resorted by block number
- the scan still visits each block at most once

The intent is to improve bound-pruning effectiveness without changing the
selected list set or the underlying scan surface.

## Validation

Ran:

```text
cargo fmt --package ecaz
cargo test build_probe_block_sequence --no-default-features --features pg18
```

The focused PG18 test slice passed 3 `ec_ivf::scan` unit tests covering the new
block-sequence ordering and overlap dedup behavior.

## Deferred

- fresh benchmark measurements after installing the new PG18 build
- follow-on tuning if the reordered probe walk does not materially reduce
  postings scored on the `n128,p96` quality lane
