## Feedback: ADR-030 v2 Review-Driven Plan Adjustment

Thank you for recording the sequencing change in durable docs (`plan/tasks/14-...`
and the ADR) rather than only in packet comments.

### What this packet does right

- Makes the gate-lift blocker list explicit: grouped scorer, cold rerank fetch,
  end-to-end recall/latency measurement, insert-path safety, vacuum-path safety,
  shared encoder contract (or equivalent proof). That list is now discoverable
  without having to reread 23 review packets.
- Keeps the architectural direction unchanged. The feedback was not a call to
  redesign; it was a call to reorder. Good judgment in absorbing that distinction.
- Interleaves rather than stacks. Packets 336-343 execute on this reordering
  (encoder contract → insert gate → vacuum gate → cold rerank fetch → stricter
  metadata → exact rerank helper → shared PQ scorer). Nothing has stalled the
  scorer runway, but the cross-cutting items are now actively being closed in
  parallel.

### One docs-level observation

The ADR now records "what blocks lifting the gate." It does not yet record
"what happens when the gate is lifted" — the operator-facing story (how does
a user opt into v2, does it become a planner decision, does it require a
rebuild, is there a migration path from v1). That's next-level design work and
not in scope for this branch, but worth a placeholder section so the question
is not rediscovered later.

### Observation

Good process outcome. The fact that the feedback was digestible enough to produce a
concrete sequencing change, rather than a defensive rebuttal, says the seams are
the right shape — otherwise the feedback would have been about the architecture,
not the order of work.
