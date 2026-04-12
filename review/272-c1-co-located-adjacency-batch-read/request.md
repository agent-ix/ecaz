# Review Request: C1 Co-Located Adjacency Batch Read

## Context

Packet `271` is now recorded as a discard: replacing transient successor `Vec`
allocations with an inline `SmallVec` did not produce a trustworthy warm win.

The current kept warm baseline from packet `270` remains:

- `p50=10.753ms`
- `p95=12.784ms`
- `p99=14.034ms`
- `mean=10.720ms`

Reviewer feedback and packet `264` still point at graph/decode overhead rather
than more scheduler container work. One specific remaining seam is that element
and neighbor tuples are often page-local, but the runtime still reads and locks
them independently.

## Problem

`src/am/graph.rs` currently loads adjacency as:

1. read/decode the element tuple
2. learn `neighbortid`
3. read/decode the neighbor tuple in a second buffer operation

That means two `ReadBufferExtended` calls and two lock/unlock cycles even when
the element tuple and its neighbor tuple live on the same page. On the scan
path, the first cache miss for a graph element in `src/am/scan.rs` pays this
cost repeatedly across many expansions.

There is already evidence that build tries to keep element and neighbor tuples
local (`pg_test_build_keeps_element_neighbor_local`), so the co-located case is
worth targeting directly.

## Planned work

1. Add a graph load helper that can decode both the element tuple and its
   neighbor tuple from one pinned page when both tuple TIDs share a block.
2. Teach the scan-local cached adjacency path to use that combined load on cache
   miss so the warm ordered-scan path benefits, not just the uncached graph
   helpers.
3. Preserve the existing fallback path for cross-page adjacency or cached
   neighbor hits.
4. Run the full checkpoint gate and rerun the verified warm `10K`, `m=8`,
   `ef_search=40`, `warm-after-prime3`, `per-cell` seam.

## Outcome

Discarded.

The implementation added a same-page adjacency decode path in `src/am/graph.rs`
and taught the scan-local cached adjacency miss path in `src/am/scan.rs` to use
that combined read on first load. The code validated cleanly, but the warm C1
surface stayed effectively flat relative to packet `270`, so the code was
reverted and nothing from this probe is kept in `main`.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurements

Canonical comparison point from packet `270`:

- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`,
  `session-mode=per-cell`, `timing-mode=cached-plan`
- baseline: `p50=10.753ms`, `p95=12.784ms`, `p99=14.034ms`, `mean=10.720ms`

Two valid reruns on the packet `272` build:

- rerun 1: `p50=10.761ms`, `p95=12.491ms`, `p99=13.396ms`, `mean=10.700ms`
- rerun 2: `p50=10.802ms`, `p95=12.741ms`, `p99=14.246ms`, `mean=10.762ms`

That is effectively flat. The first run trims the tail slightly, the second
gives it back, and the mean movement is within noise against the packet `270`
baseline.

## Decision

- revert the co-located adjacency batch-read implementation
- keep packet `272` as a recorded failed experiment
- do not land the combined locked-page adjacency decode path

## Exit criteria

- first adjacency load no longer requires two separate buffer reads when the
  element and neighbor tuple share a block
- graph/scan tests remain green
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`
  read recorded
