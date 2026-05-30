# Task 68 Timing Drilldown Summary

## Scope

- Code commit: `3471bc78bd9aea454dfa3b407d191a924b7e0447`
- Suite config: `reviews/task-68/003-timing-drilldown/artifacts/suite.json`
- Database: `task68_spire_char`
- Fixture: existing 100k corpus table from packet 002
- Index reloptions: `nlists=128`, `recursive_fanout=8`, top graph enabled,
  `storage_format='turboquant'`

## Notice Shape Fix

The emitted top-level timing fields are now disjoint for the two previously
nested areas:

- `draft_ms` is exclusive draft time.
- `draft_total_ms` keeps the outer draft wall time.
- `object_store_ms` is exclusive object-store time.
- `object_store_total_ms` keeps the outer object-store/top-graph wall time.

For the 100k rerun:

| field | ms |
| --- | ---: |
| setup | 0 |
| heap scan | 1223 |
| sample collect | 0 |
| top-level k-means | 490 |
| top-level assignment | 574 |
| recursive k-means | 1 |
| recursive assignment | 0 |
| draft exclusive | 19247 |
| top graph | 937 |
| object store exclusive | 0 |
| publish | 8 |
| total | 22482 |

Disjoint phase sum: 22480 ms, within 2 ms of `total_ms=22482`.

## Recursive K-Means Audit

The one recursive k-means call at `nlists=128`, `recursive_fanout=8` is an
actual property of the current algorithm, not an instrumentation miss:

- `recursive_routing_initial_children=128`
- `recursive_routing_final_children=8`
- `recursive_routing_iterations=1`

The loop clusters all current children directly to `target_fanout`, so 128
leaf centroids collapse to 8 routing parents in one pass, then the root object
references those 8 parents.

## Draft Drilldown

| draft field | ms | share of total |
| --- | ---: | ---: |
| `draft_input_clone_ms` | 47 | 0.2 % |
| `draft_pid_alloc_ms` | 0 | 0.0 % |
| `draft_recursive_routing_ms` | 2 | 0.0 % |
| `draft_route_map_ms` | 0 | 0.0 % |
| `draft_leaf_rows_ms` | 19182 | 85.3 % |
| `draft_leaf_inputs_ms` | 10 | 0.0 % |
| `draft_validation_ms` | 0 | 0.0 % |
| `draft_total_ms` | 19248 | 85.6 % |

The first Phase 2 P0 slice should target `build_recursive_leaf_rows_by_pid` /
boundary leaf row placement, not draft assembly broadly.

## Estimated Caps

- `draft_leaf_rows_ms` cap: 19.182 s of 22.482 s, or 85.3 % of wall time.
  Eliminating it entirely would cap the speedup at about 6.8x; a 50 % reduction
  would cap the build improvement at about 1.7x.
- `top_graph_ms` cap: 0.937 s, or 4.2 % of wall time. It is now explicitly
  separated from exclusive object-store work.
- `heap_scan_ms` cap: 1.223 s, or 5.4 % of wall time.

The ranking remains draft-first, but the actionable target is now leaf row
construction/placement within the recursive draft path.
