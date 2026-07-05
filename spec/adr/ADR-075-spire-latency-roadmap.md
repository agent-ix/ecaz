---
type: ADR
id: ADR-075
title: "SPIRE Latency Roadmap"
status: PROPOSED
impact: Affects Task 80, Task 79 follow-up evidence, SPIRE scan planning, leaf storage evolution, and future SPIRE default policy.
date: 2026-06-04
---
# ADR-075: SPIRE Latency Roadmap

## Context

Tasks 73 through 79 and the AWS 1M top-graph recall packet establish a
consistent SPIRE latency diagnosis:

- SPIRE can recover high recall at 100k and 1M.
- The high-recall path is expensive because it admits too many row candidates
  into approximate scoring.
- Final heap retention is already small relative to the scored surface.
- RaBitQ is the right primary storage-format lane versus TurboQuant, but faster
  scoring does not solve a candidate surface that is one to two orders of
  magnitude too large.
- Wider top-graph search is useful as a recall ceiling measurement, but not as
  a competitive latency recipe.

The AWS 1M top-graph recall packet made this explicit:

| nprobe | recall@10 | p50 ms | candidate sum over 500 queries |
| ---: | ---: | ---: | ---: |
| 64 | 0.9976 | 554.168 | 251,510,240 |
| 96 | 0.9994 | 779.315 | 373,897,385 |
| 128 | 1.0000 | 1038.917 | 495,000,000 |
| 256 | 1.0000 | 1029.300 | 495,000,000 |

Those rows prove that recall can be bought, but the price is too much scoring
work. SPIRE needs a durable latency roadmap so the next tasks do not drift back
to lower-impact optimizations before the candidate surface is fixed.

## Decision

Use this priority order for SPIRE latency work:

1. **Row-budgeted routing.**
   Route by an estimated row/candidate budget rather than by fixed leaf count.
   This is the nearest-term path because it attacks the fixed-nprobe failure
   without requiring leaf payload format changes if sufficient row-count
   metadata already exists or can be prototyped.

2. **Leaf-local subleaf or block pruning.**
   If selected leaves remain too large, add query-aware pruning inside selected
   leaves. This follows ADR-074: score compact per-block summaries before
   reading and scoring row payload blocks. This is likely the real long-term
   path if whole-leaf routing cannot meet the latency gates.

3. **Build geometry tuning.**
   Continue measuring `nlists`, `recursive_fanout`, top-graph frontier shape,
   and leaf density, but do not treat top-graph breadth alone as a latency
   solution. Geometry tuning is valuable when it creates smaller row units for
   row-budgeted routing or subleaf pruning to consume.

4. **Adaptive per-query nprobe.**
   Add dynamic stopping only after there is a row-budget or block-budget signal
   worth stopping on. The current fixed half-nprobe style is too blunt.

5. **Candidate scoring kernel and object layout.**
   Revisit RaBitQ scoring, batching, prefetch, and object layout only after the
   selected candidate surface is materially smaller. Kernel wins compound with
   candidate reduction, but they are not first-order while scans score hundreds
   of millions of rows.

6. **Remote and distributed path overhead.**
   Profile remote fanout, connection pooling, tuple transport, and transfer
   costs after local scan candidate volume is sane. Otherwise remote profiling
   mostly measures the wrong bottleneck.

7. **Planner and default policy.**
   Once a measured recipe exists, make SPIRE avoid bad high-recall defaults:
   cost unsafe combinations accurately, cap pathological settings, or add an
   explicit quality/latency preset. Do not ship policy before the candidate
   surface has a defensible recipe.

Task 80 owns the first two items. Later tasks should reference this ADR when
they pick up items 3 through 7.

## Rationale

The measured bottleneck is the row candidate surface. Work that does not reduce
that surface is unlikely to produce a durable high-recall latency win.

Row-budgeted routing is first because it can use the existing recursive routing
shape and should produce immediate evidence about whether whole selected leaves
are a sufficiently fine unit. Subleaf pruning is second because Task 79 already
showed whole-leaf routing nearly reaches the candidate gate but remains too
coarse. The rest of the roadmap is sequenced around those facts:

- geometry matters because it controls leaf size and routing granularity;
- adaptive nprobe needs a meaningful stop condition;
- scoring kernels matter after there are fewer rows to score;
- distributed overhead matters after local scoring is not dominant;
- defaults should encode a proven recipe, not a hopeful one.

## Alternatives Considered

### Continue widening top-graph search

Rejected as a latency strategy. The AWS 1M run recovered recall only by
expanding the candidate surface to `251M` through `495M` candidates over 500
queries.

### Focus on scoring micro-optimization first

Deferred. RaBitQ already improved the storage-format lane relative to
TurboQuant, but Task 77 and Task 78 showed scoring remains dominant because too
many candidates reach the scoring loop. A 20% scorer win does not close a 27x
to 54x candidate-surface problem.

### Tune defaults before new scan behavior

Rejected. Task 76 found no default point that beats current defaults once
high-recall latency and tails are considered. Defaults should follow a measured
candidate-selection win.

### Treat remote/distributed overhead as the next bottleneck

Deferred. It is real, but the latest 1M evidence is dominated by local candidate
volume. Remote-path profiling is useful after local candidate budgets are under
control.

## Consequences

### Positive

- Keeps SPIRE latency work aligned with measured bottlenecks.
- Separates near-term row-budget routing from deeper leaf format work.
- Prevents top-graph breadth and default-policy work from being mistaken for
  latency fixes.
- Gives future scoring-kernel and distributed-path tasks a clear prerequisite.

### Negative

- Row-budgeted routing may require metadata that is not currently persisted in
  the ideal location.
- Subleaf pruning likely requires a leaf format bump and build-cost work.
- The roadmap delays some easier-looking constant-factor optimizations until
  the candidate surface is smaller.

## Acceptance Criteria

Task 80 should close the first phase of this roadmap by proving one of:

- row-budgeted routing reduces scored candidates at matched recall and lands as
  the first latency fix;
- row-budgeted routing is insufficient because selected leaves remain too
  coarse, and ADR-074-style subleaf pruning lands or is split into a deeper
  format task with evidence;
- neither path is viable under current SPIRE geometry, and a follow-up task
  owns the required build/format redesign.

Any future task that claims to advance SPIRE latency should state which roadmap
item it addresses and cite packet-local evidence for candidate count,
recall@10, p50/p95/p99 latency, and retained/returned rows.
