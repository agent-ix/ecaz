# Task 30 Phase 9: SPIRE Graph Architecture

Status: proposed
Owner: coder1 / SPIRE graph track
Priority: 1

## Goal

Turn the current locally functional SPIRE hierarchy into a graph/routing
architecture that can scale compositionally across many hierarchy levels.

Phase 9 is about the shape of the index and routing plan, not remote transport
throughput. It should resolve the architectural findings from
`review/30658-spire-phase9-routing-plan/feedback/2026-05-09-01-reviewer.md`
before Phase 10 optimizes execution.

## Scope

- Decouple top-graph routing from root fanout.
- Define the top-graph frontier explicitly.
- Add global route budgets so recursive routing cannot grow multiplicatively.
- Make multi-level routing diagnostics explainable and measurable.
- Land the global vector identity contract needed for boundary replicas and
  multi-instance dedupe.
- Keep RaBitQ as the supported first compact scoring format. PQ/PQFastScan
  remains out of scope unless a slice is only cleaning up unsupported option
  wording.

## Entry Rules

- The Phase 8 AWS/RDS-class scale packet remains the gate for product-scale
  claims.
- Phase 9 architecture work may proceed without that external performance run
  when the operator explicitly waives the scale gate for design work. Local
  PG18 evidence proves functionality only, not production performance.
- Each implementation slice needs its own review packet and narrow validation.

## Phase 9.1: Top-Graph Frontier Contract

- [x] Decide what the top graph is built over: ADR-054 chooses the active
  root/top routing object's child frontier. The future scale build must make
  that root/top child set large enough for graph routing instead of compressing
  it down to `recursive_fanout`.
- [x] Record the decision in an ADR or design note if the selected frontier is
  not already covered by `plan/design/spire-top-level-graph.md`. Recorded in
  `spec/adr/ADR-054-spire-top-graph-frontier-contract.md`.
- [x] Update manifest/diagnostic terminology so operators can distinguish
  root fanout, graph node count, routing level, and leaf count. The top-graph
  snapshot now exposes frontier kind, parent/child levels, frontier node count,
  root child count, and active leaf count.
- [x] Add validation that rejects ambiguous graph/root mismatches with an
  actionable error. Strict scan validation already rejects graph/root shape
  mismatches; the top-graph snapshot now also reports root, level, or frontier
  mismatch statuses instead of calling such graphs ready.

## Phase 9.2: Scalable Top-Graph Storage

- [x] Remove the single-tuple top-graph storage ceiling.
- [x] Choose one storage shape: reuse the relation-object V2 chain format for
  routing and top-graph objects, with generic partition-object chain codecs.
- [x] Preserve epoch/placement validation: graph segments must be visible only
  through the active manifest and must not be read outside their epoch.
- [x] Add diagnostics for graph byte size, segment count, node count, degree,
  and build/search list sizes.

## Phase 9.3: Cached / Borrowed Graph Routing

- [ ] Stop rebuilding a `VamanaGraph` by copying all neighbor lists on every
  query.
- [ ] Add a borrowed graph view or cached scan/relcache structure for
  top-graph adjacency.
- [ ] Avoid the full query-to-centroid offset scan if routing can compare
  monotonic `-inner_product` scores directly.
- [ ] Keep deterministic tie-breaks visible in tests and diagnostics.

## Phase 9.4: Global Recursive Beam

- [ ] Replace per-parent independent top-N expansion with a scored global
  frontier.
- [ ] Add explicit scan controls:
  - `beam_width`;
  - `max_leaf_routes`;
  - `max_routing_expansions`;
  - optionally `max_candidate_rows` if Phase 10 has not landed it first.
- [ ] Dedupe leaf routes before storage reads.
- [ ] Expose routing diagnostics per level: input frontier width, expanded
  parent count, selected child count, deduped route count, and truncation
  reason.
- [ ] Keep existing `nprobe_per_level` as a local per-parent or per-level
  budget input, but make the global beam the final guardrail.

## Phase 9.5: Boundary Replication Execution Contract

- [ ] Finish the runtime contract for the existing boundary-replica build path:
  primary row, replica row, assignment flags, route selection, and dedupe.
- [ ] Define how replicas interact with top-graph/frontier routing. A query
  should not need to know whether a candidate came from a primary or replica
  placement until merge tie-breaks.
- [ ] Add recall/storage diagnostics that separate primary rows, replica rows,
  duplicate candidates suppressed, and candidate winners.
- [ ] Keep boundary replication opt-in until recall and storage overhead are
  measured.

## Phase 9.6: Global Vector Identity

- [ ] Define the durable global `SpireVecId` format for multi-node search.
- [ ] Decide whether the identity is:
  - coordinator-assigned global ID;
  - node-id plus local sequence;
  - original-vector ID plus serving-placement metadata; or
  - another stable encoded form.
- [ ] Ensure boundary replicas share the same original-vector identity for
  dedupe, even when stored in different leaves or on different nodes.
- [ ] Update remote merge preconditions so multi-node callers cannot silently
  dedupe unrelated node-local IDs.
- [ ] Add migration/compatibility behavior for existing local-only IDs.

## Phase 9.7: Quality Experiments

These remain below the structural graph work. Do not start them until
top-graph frontier, global beam, and identity contracts are stable.

- [ ] IMI reshape of centroid/routing storage for A/B comparison.
- [ ] Adaptive `nprobe` or adaptive beam policy.
- [ ] Anisotropic centroid scoring as the headline quality target.
- [ ] Query difficulty estimator stretch.

## Validation

- Use focused PG18 tests for routing invariants, manifest validation, and scan
  behavior.
- Use `git diff --check` for docs-only planning updates.
- Measurement claims need packet-local artifacts and a manifest.
- Do not claim product-scale performance until the Phase 8 scale packet is
  complete or explicitly waived for that claim.

## Exit Criteria

- Top graph can be larger than root fanout and larger than one tuple.
- Recursive routing has a global route budget with diagnostics.
- Leaf routes are deduped before I/O.
- Boundary-replica dedupe relies on a stable vector identity.
- Phase 10 can optimize execution without changing graph semantics again.
