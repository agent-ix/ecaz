# Feedback: Consumed-Candidate Refill and Benchmark Baseline

Request:
- `review/59-consumed-candidate-refill-and-benchmark-baseline.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus Areas

### Scan correctness: consumed-candidate refill contract

The refill contract is correct. After consuming the frontier head, `refill_bootstrap_frontier_after_consume` (scan.rs:644-662) expands from the consumed candidate's neighbors if it hasn't been expanded before, then tops up from the best unexpanded frontier candidate. The visited set prevents re-scoring, and the expanded-source set prevents re-expanding. This matches the intended traversal groundwork.

The frontier compaction semantics are sound: `Vec::remove(head)` shifts elements, then `recompute_candidate_frontier_head` re-derives the best index. The refill may grow the Vec back toward `MAX_BOOTSTRAP_FRONTIER_CANDIDATES`, or it may not (if no unseen neighbors exist). Both outcomes are handled — the frontier simply operates at whatever size it reaches.

One edge worth noting: if all frontier candidates have been expanded and the frontier shrinks below `MAX_BOOTSTRAP_FRONTIER_CANDIDATES`, the `top_up_bootstrap_frontier` loop (scan.rs:515-524) will break immediately because `next_bootstrap_expand_index` returns `None`. This is correct — there's nothing left to expand from.

### Benchmark surface: bench_api narrowness

The `bench_api` exports in `src/lib.rs` are the right approach — a thin re-export surface that doesn't leak internal structure. The scoring and decode benchmarks measure the actual entry points used during scan (particularly `score_ip_from_parts` which is the real hot path). The criterion benchmarks should pre-generate all test data outside timed closures, which the review's methodology constraints correctly mandate.

### Coverage quality: SRHT and wrapper-equivalence tests

The SRHT real-world dimension proptest (384, 768, 1024, 1536) is a high-value addition — these dimensions exercise the power-of-2 padding path that production workloads hit. The `score_code_inner_product` equivalence test confirms the SQL-surface scorer matches the internal quantizer path, which is a clean boundary test.

These are the right low-risk additions before broader recall-data realism work.

## Additional Findings

No issues found.
