---
id: ADR-054
title: "SPIRE Top-Graph Frontier Contract"
status: ACCEPTED
impact: Affects ADR-049, Task 30 Phase 9 graph architecture
date: 2026-05-09
---
# ADR-054: SPIRE Top-Graph Frontier Contract

## Status

Accepted.

## Context

The first SPIRE top-graph implementation validated storage and routing by
building a Vamana graph over the active root routing object's children. That
works as a local proof, but the current recursive builder also compresses until
the root has at most `recursive_fanout` children. A graph over that small root
fanout is not the SPIRE paper's large top-level routing graph; it is only a
small accelerator over an already-compressed root.

Task 30 Phase 9 needs a durable contract before storage and query execution are
expanded. The contract must distinguish:

- root/top routing object fanout;
- graph node count;
- routing parent level and graph child frontier level;
- total active leaf count;
- configured recursive fanout.

Without those terms, the implementation can report "top graph ready" while the
graph is still structurally tied to a tiny root fanout.

## Decision

The SPIRE top graph is built over the children of the published root/top
routing object for the active epoch. That root/top object owns the graph
frontier, and its child set is the graph node set.

This means the long-term scale shape is not "compress recursively until the
root has `recursive_fanout` children, then graph those children." Instead, the
build coordinator must stop recursive compression at an explicit top-graph
frontier and publish a root/top routing object whose children are the frontier
nodes. The graph then searches that child set and hands selected child PIDs to
the lower recursive descent.

The current root-child implementation remains a valid compatibility shape when
the root child set is small. It must be described as
`root_top_routing_children`, not as a product-scale top graph by itself.

## Required Invariants

For a ready top graph in an active epoch:

- exactly one visible top-graph object exists;
- exactly one visible root routing object exists;
- the top graph's `root_pid` equals the active root routing PID;
- the top graph's parent level equals the root routing object's level;
- the graph node count equals the active root routing object's child count;
- each graph node maps to the corresponding root/top routing child PID and
  centroid ordinal;
- scans fail closed for graph/root mismatches in strict mode.

Diagnostics must expose enough fields to spot accidental root-fanout-bound
graphs:

- frontier kind;
- frontier parent level;
- frontier child level;
- frontier node count;
- root child count;
- active leaf count;
- graph node count.

## Rationale

This preserves the existing graph object model while removing the implicit
coupling between useful graph size and `recursive_fanout`. It also keeps lower
recursive routing unchanged: graph search chooses a bounded top-level frontier,
then existing level-by-level descent continues from those selected child PIDs.

The alternative "graph over all internal routing objects at some level while
the root remains tiny" would require a second parent namespace and ambiguous
PID ownership rules. Making the root/top object own the graph frontier keeps
epoch validation, manifest lookup, and strict scan failure behavior local to
the existing SPIRE object model.

## Consequences

- Phase 9 storage work must remove the single-tuple graph object ceiling before
  large frontiers are possible.
- Phase 9 build work must add an explicit top-graph frontier stop condition
  instead of always compressing to `recursive_fanout`.
- Phase 9 diagnostics must avoid treating graph node count and recursive root
  fanout as the same concept.
- Phase 10 execution work can optimize remote fanout and candidate streaming
  without changing what graph nodes mean.

## Open Questions

- Which user-facing option should select the top-graph frontier size or level?
- Should the build coordinator choose the frontier by target node count,
  routing level, memory budget, or measured routing quality?
- Should a future graph object duplicate centroid vectors for faster query
  routing, or keep joining graph nodes back to the root/top routing object?
