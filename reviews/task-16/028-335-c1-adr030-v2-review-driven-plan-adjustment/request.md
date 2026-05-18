# Review Request: C1 ADR-030 V2 Review-Driven Plan Adjustment

## Context

Reviewer feedback is now present on every ADR-030 packet from `310` through `333`.

The feedback does **not** overturn the ADR-030 v2 design:

- transformed grouped `PQ4` search code
- hot binary sidecar
- cold rerank payload
- `binary -> grouped -> rerank` query pipeline

But it does clarify that the current execution order was starting to bias too heavily toward
scorer-shaping packets while several cross-cutting safety items were only being tracked in review
notes.

## Problem

If the branch keeps pushing only on scorer seams, several important requirements would pile up near
the end:

1. insert-path grouped-v2 safety
2. vacuum-path grouped-v2 safety
3. shared grouped encoder contract
4. cold rerank fetch
5. stronger runtime metadata validation
6. full end-to-end recall measurement

That is the wrong place to discover them. The review feedback is clear that these items do not
block the current experimental lane, but they do block lifting the experimental gate.

## Planned Adjustment

Record the feedback-driven sequencing change in the durable planning docs:

1. keep the v2 architecture unchanged
2. keep moving toward the real grouped scorer
3. interleave correctness/safety items before the scorer lane gets too far ahead
4. make gate-lift blockers explicit in task/ADR text

## Implementation

Updated:

- `plan/tasks/14-adr030-v2-grouped-index.md`
- `spec/adr/ADR-030-fastscan-grouped-subvector-scoring.md`

Recorded changes:

1. marked the already-landed feasibility / metadata / build slices as complete in the task file
2. added a feedback-driven reordering section to the task file
3. made the following explicit as immediate interleaved work:
   - shared grouped encoder contract
   - insert/vacuum format safety
   - cold rerank fetch smoke path
   - then the real grouped scorer
4. made the following explicit as experimental gate-lift blockers:
   - grouped scorer
   - cold rerank fetch
   - end-to-end recall/latency measurement
   - insert-path safety
   - vacuum-path safety
   - shared encoder contract or equivalent proof
5. added the same sequencing update to ADR-030 so the rationale is durable outside the task list

## Measurements

Docs/planning-only adjustment.

No runtime or recall measurements were added in this packet.

Known validation results for this attempt:

- reviewed feedback files for packets `310-333`
- updated ADR/task planning docs only
- no code changed
- no checkpoint test suite was run for this packet

## Outcome

ADR-030 v2 now has a review-driven execution order recorded in-repo rather than only implied by
packet comments.

The key outcome is sequencing:

1. the grouped scorer is still the central runtime milestone
2. but insert/vacuum safety, cold rerank fetch, and encoder-contract cleanup are now explicitly
   interleaved rather than deferred to the end
3. the branch now has an explicit list of what blocks lifting the experimental build gate

## Next Slice

The next implementation slice should reflect the reordering above, not the old "scorer only" bias.

The strongest immediate candidates are:

1. shared grouped encoder contract
2. grouped-v2 insert-path rejection
3. grouped-v2 vacuum-path rejection
4. cold rerank fetch smoke seam

The actual choice should prefer whichever removes the highest-risk ambiguity before the first real
grouped scorer packet lands.
