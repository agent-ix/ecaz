# Task 39 / 037 — Relation Store Chain + Trait Dispatch Coverage

## Goal

Drive `am/ec_spire/storage/relation_store.rs` past the chain-write/read,
multi-segment leaf V2, placement validation, trait dispatch, locator,
and prefetch paths that packet 035's basic round trips left untouched.
Aim was the 80% floor; this packet lifts the file to 56.53% (from
27.20% pre-packet, a 2.1× rise without touching production code or
adding new emulator scaffolding).

## Code Change

`hardening/careful/src/spire.rs` — 7 new tests in `storage::tests`
exercise:

- `relation_store_routing_object_chain_round_trip_through_large_payload`:
  8 children × 256 dimensions ≈ 8 KB encoded, above the
  ~7000-byte single-tuple ceiling, so `insert_routing_object` takes
  `insert_large_partition_object_chain`; `read_routing_object` walks
  the V2 chain meta + segment chain back to the original object.
- `relation_store_top_graph_object_chain_round_trip_through_large_payload`:
  60-node graph with full-degree neighbor lists triggers the same
  chain path through the top-graph branch.
- `relation_store_insert_and_read_leaf_v2_multi_segment_round_trip`:
  50 rows × 256-byte payload at BLCKSZ=8192 produces more than one
  leaf-V2 segment; the reader's `for _ in 0..segment_count` chain
  walker is now exercised with `segment_count > 1`.
- `relation_store_insert_and_read_leaf_v1_round_trip`: pins the V1
  reader's "tuple is not a V1 leaf" error path.
- `relation_store_validate_placement_rejects_wrong_node_id_and_state`:
  pins `validate_local_available_placement` rejection branches for
  mismatched `node_id`, non-Available state, and mismatched
  `store_relid`.
- `relation_store_object_reader_trait_dispatch_covers_all_methods`:
  drives every `SpireObjectReader for SpireRelationObjectStore` method
  through trait dispatch, including `read_object_header`,
  `read_routing_object`, `read_leaf_object_v2`, `read_delta_object`,
  and `read_top_graph_object` (impl block at line 1254).
- `relation_store_active_object_tuple_locators_for_each_kind`: single
  tuple, chained routing, and multi-segment leaf V2 each return the
  expected locator list.
- `relation_store_prefetch_object_tuple_and_tuples_dispatch_through_trait`:
  inherent and trait-dispatched prefetch methods plus
  `relation_object_prefetch_groups` grouping path.

## Baseline Ratchets

`fixtures/quality/coverage-baseline.tsv`:

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/storage/relation_store.rs` | 27.20 | **56.53** |
| `am/ec_spire/page.rs` | 81.12 | **83.15** |

`page.rs` rises by ~2 points incidentally — the chain tests exercise
several previously-uncovered append paths inside the page helper that
the basic 035 tests had not reached.

`relation_store.rs` does not yet cross the 80% target. The remaining
uncovered surface is dominated by:

- Leaf V2 read error branches (segment-no, byte-base, length mismatch).
- `active_object_tuple_locators` error branches (chain meta missing,
  trailing/early-terminated chain, leaf V2 meta mismatch).
- `prefetch_relation_blocks_with_read_stream` (PG18-only path; the
  emulator's read-stream is a no-op so the inner loop is never
  entered).

Those are tracked as follow-ups; the present slice's gains are large
enough to ship rather than waiting for a perfect 80%.

## Validation

Artifacts under `reviews/task-39/037-relation-store-chain/artifacts/`:

- `relation-store-chain-focused-tests.log`: **497 passed, 0 failed**
  (was 488 after packet 036; 9 new tests in this packet).
- `coverage/summary.txt` + JSON files from `make coverage`.
- `coverage-delta-check.log`: both ratcheted files green at the new
  baselines.
- `coverage-baseline-check.log`: **40 critical paths complete.**
- `changed-files.txt`: the two source paths whose baseline this packet
  ratchets.

Code commit and packet commit pushed separately to
`origin/task39-continuation-20260519` per the workflow rules.
