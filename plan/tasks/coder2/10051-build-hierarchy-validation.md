# Task: Build-Time Hierarchy Validation

Motivation: Review 212 found that `flush_build_state` was iterating only layer 0
from hnsw_rs, collapsing the entire HNSW hierarchy to level 0. All 10,000 nodes
were persisted as level=0. This was the dominant remaining recall bug. This task
adds validation to catch that class of bug at build time.
Priority: batch 2
Status: ready

## Prompt

Add a build-time validation in `flush_build_state` (`src/am/build.rs`, line 523)
that catches hierarchy collapse immediately.

After `build_hnsw_graph` returns `graph_nodes` (line 530) and before the element
insertion loop (line 532), add a debug assertion block that verifies:

1. The max level across `graph_nodes` is > 0 when there are enough nodes for a
   multi-layer hierarchy (e.g., > 2*m nodes). For m=8 and 10k nodes, hnsw_rs
   should produce max_level of 3-5. A max_level of 0 means the hierarchy
   collapsed.

2. The level distribution is not degenerate: not all nodes should be at the same
   level.

Use `debug_assert!` so this is free in release builds but catches regressions in
test runs:

```rust
#[cfg(debug_assertions)]
{
    let max_level = graph_nodes.iter().map(|n| n.level).max().unwrap_or(0);
    let node_count = graph_nodes.len();
    if node_count > 2 * state.options.m {
        debug_assert!(
            max_level > 0,
            "tqhnsw build produced a flat hierarchy (max_level=0) with {} nodes and m={}; \
             this likely means the build is not reading upper-layer assignments from hnsw_rs",
            node_count,
            state.options.m
        );
    }
}
```

Also add a test-gated log line that prints the level distribution summary during
build, gated behind `#[cfg(any(test, feature = "pg_test"))]`. Use `pgrx::debug1!`
or `pgrx::info!`. Format like:

```
tqhnsw build: 10000 nodes, max_level=4, level distribution: [9600, 350, 40, 8, 2]
```

This helps diagnose hierarchy issues without needing a separate SQL probe.

Insert after the `build_hnsw_graph` call (line 530) and before the element
insertion loop (line 532).

## Validate

```bash
cargo test
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Branch from current upstream main. Push branch for review.
