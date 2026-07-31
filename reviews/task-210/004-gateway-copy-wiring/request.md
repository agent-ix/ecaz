# Task 210 P3 — TRAV-30 gateway copy wiring

- Branch: `task-203-ec-distann-conformance`
- Code commit: `600f71985` (`feat(ec-distann): wire TRAV-30 gateway copies
  into the physical read path`)
- Primitive under wire: `gateway_copy.rs` (`c7ee0d9ba`, packet-less P3 landing
  noted in the Task 210 handoff §3)

## What changed

The bounded gateway copy set is now populated and consulted; before this
commit nothing read `ec_distann.gateway_copy_capacity`.

1. **Population** (`populate_gateway_copies`, `generation_read.rs`): once per
   cached physical epoch, when `gateway_copy_capacity > 0` and the roster is
   multi-node, the coordinator copies the routing payload of the FR-080 head
   landmarks — neighbour ids + neighbour codes only, bounded by the GUC
   capacity, never a full-precision vector. Local landmarks are read in
   process (`resolve_nodes`); remote landmarks through the new bounded
   `ec_distann_gateway_routing_export(regclass, bytea, bigint[])` owner
   endpoint. Same source as the FR-084 replica stream's `graph_record` half,
   bounded destination — that difference is TRAV-30. Population failure
   degrades to no-gateway **with a warning** (never a scan failure, never
   silent).
2. **Serving** (`PhysicalMultiOwnerExpander::expand_nodes_raw`): ids with a
   gateway copy still go to their owner — `exact_dist` needs the owner's
   co-placed vector (Algorithm 1's *result* half; caching vectors at the
   coordinator is exactly the FR-084 trap, handoff §4c) — but the request
   names them in `skip_neighbor_vec_ids` and the owner returns them with an
   empty neighbour payload and no scoring work
   (`GenerationExpander::expand_nodes_masked`). The coordinator reconstructs
   the *candidate* half locally from the copy (same codes, same
   `score_dists_batch`, same per-row threshold).
3. **Batch semantics preserved exactly** (Task 205): the owner applies the
   batch L limit to the uncached subset; the coordinator refills cached rows
   and re-applies L over the whole owner batch in original row positions.
   `top-L(top-L(uncached) ∪ cached) == top-L(full batch)` under the
   deterministic `(dist, vec_id, row, idx)` tie-break. Pinned by
   `gateway_fill_and_rebatch_matches_the_owner_only_batch_semantics`, which
   includes a duplicated equal-score neighbour across a cached and an
   uncached row.
4. **FR-079-AC-1 preserved**: one response row per requested id, in request
   order; a cached row differs only in an empty neighbour payload.
   `place_physical_owner_responses` is unchanged.
5. **Activation is observable, not inferred** (handoff §6.1): per-backend
   counters via the new `ec_distann_gateway_copy_stats()` →
   `(capacity, entries, resident_bytes, served)`, plus a
   `gateway_copies_served` benchmark stage counter. A run that claims the
   mechanism is active must show `served > 0`; `fill_gateway_rows` also
   hard-errors if an owner returns a neighbour payload for a row it was told
   to skip (the mechanism cannot silently not-happen on the wire).

## NFR-021 position

Nothing vector-shaped is cached: `DistannGatewayCopy` holds
`(vec_id, is_tombstone, neighbor_vec_ids, neighbor_codes)`. Capacity is a
stated constant enforced by refusal, `resident_bytes` is what the
conformance emitter can report, and the copy is epoch-scoped and rebuildable.
P3 is judged on **response bytes and owner scoring work**, not eliminated
hops — every expansion still pays its owner round trip.

## Validation

- `artifacts/gateway-copy-unit-tests.log` — 5/5 gateway tests pass,
  including the batch-semantics equivalence test.
- `cargo test --lib am::ec_distann`: 185 pass; 2 failures are not this
  change: the quantizer SIMD width test is the pre-existing failure recorded
  in the handoff §7 (reproduces with Task 210 changes stashed), and the
  `scan_registry` lock-release test passes when run alone (parallel-test
  flake).
- `cargo clippy --no-default-features --features pg18 -- -D warnings` and the
  `pg18 + distann-head-attribution-benchmark` variant: clean. (The
  `--all-targets` clippy gate currently fails on a `needless_range_loop` in a
  `head_sample.rs` **test** from the P2a slice — not this commit's code;
  flagged for the P2 owner.)

## Owed / not in this packet

- **A/B bench evidence** (bytes + owner work + recall/latency at
  10/50/100k, gateway on vs off). The bench host is occupied by the P2
  head-sharding A/B; the fixture also needs a `--gateway-copy-capacity`
  flag to make the arm suite-addressable (parallel to `--sharded-head`,
  `862c03547`). The task stays open until that evidence lands — this packet
  requests review of the wiring, not closeout.
- Multinode PG18 behaviour validation rides the same future run.
