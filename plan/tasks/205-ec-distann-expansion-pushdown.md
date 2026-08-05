# Task 205: ec_distann Expansion Pushdown (Paper Algorithm 1)

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **ready** (2026-07-29). Priority: P0 latency prerequisite.

Entry gate: none technically, but land Task 204 first if both are in flight so
the A/B's storage rows are trustworthy.

## Why

`DISTRIBUTEDANN` §2.3 Algorithm 1 runs on each storage host and receives a
threshold score `t` and a candidate limit `l`; §2.4 supplies
`t = peek_worst(H_C)` each round; the host prunes and truncates **before**
returning. Equation (2)'s ~6x score-vs-node bandwidth saving is already banked
by ec_distann's score-only wire format; the paper says threshold pruning
increases that saving further without quantifying the incremental factor.

Task 203 found none of this is live:

- `code_threshold` is hardcoded `None` at the sole orchestration call site
  (`src/am/ec_distann/scan.rs:215`). No production or test site passes `Some(..)`.
- The production physical expander discards it entirely
  (`generation_read.rs:3146-3149`, `_code_threshold`); so does the replica
  (`traversal_replica.rs:2455`). Only the legacy `LocalNodeExpander`
  (`expand.rs:127-137`) honors it, and it is reachable only with `None`.
- Candidate limit `l` does not exist in FR-079, FR-081, any struct, or any SQL
  overload.
- `peek_worst(H_C)` has no counterpart. `scan.rs` reads only its beam's *best*
  unvisited member, for the convergence early-exit.
- Owners return every neighbor of every requested node, unsorted and untruncated.

`FR-079:115-123` defaulted the threshold off deliberately, resolving
failure-domain finding FND-006. That is defensible on its own terms; what was
never recorded is the coupling — it removes the mechanism that makes the paper's
beam width affordable. This is why Tasks 188 and 194 measured beam widening as a
transport cost rather than a saving, and why `TRAV-14`/`TRAV-15` are void.

## Goal

Implement Algorithm 1's pushdown in the **production** expansion path and measure
it in isolation at the current BW=4/H=100, so its effect is attributable
separately from any beam-width change.

## The recall-equivalence argument

FR-079 currently frames `code_threshold` as an optimization that "may prune true
results". The paper's threshold is different in kind: it is derived from the
coordinator's **own** candidate heap, so a neighbor failing it is one the beam
would have discarded on arrival. That makes result-equivalence plausible rather
than a recall risk.

**This must be established, not assumed.** Requirements:

- a stated argument for why `t = peek_worst(H_C)` cannot prune a candidate that
  would have entered the retained heap, including the interaction with `L`, the
  early-exit bar, and tombstones whose `exact_dist` is NULL;
- tests proving ordered-result identity against the `None` path on the same
  generation, including ties, tombstones, and mixed-owner frontiers;
- an explicit statement of what breaks the equivalence (e.g. a threshold taken
  from a stale round, or `l` smaller than the beam can consume).

If equivalence cannot be established, the feature ships as opt-in and labeled,
and the task says so rather than weakening FR-081-AC-4.

## Phases

1. **Spec slice.** Amend FR-079 for candidate limit `l`, the owner-side
   prune/sort/truncate contract, and the coordinator-derived threshold with its
   equivalence claim. FR-081 records the per-round `t` derivation.
2. **Coordinator.** Derive and pass `t` and `l` per round from the live candidate
   heap. Remove the hardcoded `None`.
3. **Owner.** Implement threshold filtering followed by one merged-batch
   partial-sort + truncate at `l = L` in the production physical expander.
   Keep the legacy path consistent. The selected BW=4/H=100 default is
   `L=32`: it satisfies `L >= max(BW, k)` for the k=10 gate, makes the
   threshold observable against degree 32, and is intentionally regime-sized
   rather than copied from the paper's BW=128/L=200 setting.
4. **A/B at fixed BW=4/H=100** so the pushdown is attributable on its own.

## Benchmark gate

`ecaz bench suite` A/B at 10k/50k/100k, recall + latency + storage, with the
**owner-traversal arm as control** (NFR-022) and an NFR-021 admissibility verdict
recorded at pre-registration. Report request/response bytes and per-round
transport wait alongside end-to-end latency: the paper's claim is a bandwidth
saving, so bytes moving is the mechanism check, and end-to-end latency is the
decision.

## Required review packets

1. `reviews/task-205/001-contract/` — FR-079/FR-081 amendment and the
   equivalence argument.
2. `reviews/task-205/002-implementation/` — code checkpoint plus focused PG18 and
   result-identity evidence.
3. `reviews/task-205/003-ab/` — the 10k/50k/100k matrix and the decision.

## Non-goals

- Changing BW, H, `top_k`, or seed count. That is Task 206, and stacking them
  destroys attribution.
- Head changes (Task 207).
- Reviving the traversal replica.

## References

- `DISTRIBUTEDANN` §2.3 (Algorithm 1), §2.4 (Algorithm 2), equation (2). The
  incremental threshold effect is judged as an unquantified response-byte
  reduction, not as the paper's already-banked ~6x score-only saving.
- `reviews/task-203/001-decision-reaudit/` Defect 2.
- `FR-079`, `FR-081`, `NFR-019`, `NFR-021`, `NFR-022`; `spec/reviews/failure-domain.md:40`.
