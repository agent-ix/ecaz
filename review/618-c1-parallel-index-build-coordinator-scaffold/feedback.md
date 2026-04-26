# Feedback: 618 Parallel Index Build Coordinator Scaffold

## Verdict: Accept

Scaffold is the right shape. The plan enum explicitly models every axis of the
build decision tree before any executable code lands, which is the correct
approach for a new coordination boundary.

## Findings

**Coordinator boundary**: `build_parallel.rs` as a dedicated module is
correct. The scan coordinator in `common/parallel.rs` is shaped around
descriptor attachment, rescan epochs, and traversal snapshots — none of which
apply to a build. Starting fresh avoids pulling scan-shaped constraints into
the build path.

**Plan enum design**: Modeling `CoordinatorKind`, `HeapIngest`, `TupleSink`,
and `GraphAssembly` as separate axes is good. It makes future additions (e.g.,
shared sorter for `TupleSink`, parallel graph for `GraphAssembly`) explicit
changes to a named field rather than implicit behavior branches.

**`leader_participates = true` in scaffold**: This gets corrected to `false`
in packet 619 because the leader is dedicated to queue draining. The scaffold
value is not wrong at this stage — it's a placeholder — but it does create a
test in packet 618 that asserts `participant_count = 4` for 3 workers, which
packet 619 then invalidates. Not a problem since 619 updates the test.

**SharedHeader atomics**: Using `AtomicU64` for `scanned_heap_tuples` and
`encoded_index_tuples` is correct for shared-memory counters. These get
replaced by non-atomic fields protected by the mutex in packet 619 when the
counter update path is worker-sequential (leader accumulates after each worker
finishes). Both designs are defensible.

**`amcanbuildparallel = false`**: Correct gating point. PostgreSQL should not
plan parallel index builds until the worker entrypoint and DSM are executable.

**Tests**: Plan tests and shared-header accumulation test cover the right
properties at this stage.

## No Issues
