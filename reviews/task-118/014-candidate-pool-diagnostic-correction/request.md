---
task: 118
packet: reviews/task-118/014-candidate-pool-diagnostic-correction
checkpoint_sha: df7ff2a0324929bd385e710ed97807be971773df
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Candidate-Pool Diagnostic Correction

## Scope

This checkpoint corrects the packet 012 frontier diagnostic interpretation.

The scan-internal visible frontier is too early in the graph-first scan
lifecycle: result staging happens through gettuple, so sampling that vector
after rescan produced empty `frontier_*` arrays at runtime. Task 118 needs the
AM's ef_search-sized candidate pool before SQL LIMIT truncation. Commit
`df7ff2a0324929bd385e710ed97807be971773df` therefore reports the full
AM-emitted candidate stream as the pre-final frontier/candidate pool.

Code changes:

- remove the empty visible-frontier capture path from
  `debug_gettuple_frontier_containment_report`;
- derive `pre_final_frontier_size` and `frontier_candidates` from the full
  AM-emitted candidate stream;
- update the focused pg_test to assert the candidate-pool diagnostic is
  non-empty and internally consistent.

Handoff updates:

- packets 010 and 011 now point at packet 014 for the corrected diagnostic
  semantics;
- packets 012 and 013 now carry supersession notes so reviewers do not treat
  the empty visible-frontier interpretation as final.

## Validation

- `cargo check --features 'pg18 pg_test' --no-default-features`
  - Artifact: `artifacts/cargo-check-pg18-pgtest.log`
  - Result: passed
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
  - Artifact: `artifacts/cargo-pgrx-install-pg18-pgtest.log`
  - Result: passed
- 10k TurboQuant source-build smoke, `ef_search=200`, `queries_limit=5`
  - Artifact: `artifacts/frontier-smoke-10k-turboquant-ef200.log`
  - Artifact: `artifacts/frontier-smoke-10k-turboquant-ef200-summary.md`
  - Result: all five rows reported `pre_final_frontier_size=200`,
    `frontier_row_count=200`, `final_emitted_count=200`, and
    `truth_top10_in_frontier=10`.

The smoke ran on the slower AMD host and is not final benchmark closeout
evidence. It proves the corrected diagnostic surface is populated before the
Intel 10k/50k/100k closeout runs.

## Remaining Task 118 Closeout Work

Regenerate the 10k frontier diagnostics on current head for all source and
compressed-build lanes, then run the Intel 50k/100k suites and update packet
006 with the final dominant-loss classification.
