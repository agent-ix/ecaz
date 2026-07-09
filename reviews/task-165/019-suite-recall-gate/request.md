# Review request — Task 165: suite-driven recall gate (006-P1) + a real finding

**Branch:** `task-165-ec-distann-m3`. Wires the `ecaz bench recall` form of the
multinode recall gate (006-P1's letter) into the fixture — and it immediately
earned its keep by catching a **real multi-node recall gap at higher top_k** that
the default-top_k byte-identical proof missed.

## What landed

1. **Bench-runner fix** (`crates/ecaz-cli/src/commands/bench/mod.rs`):
   `apply_session_gucs` now uses parameterized `set_config($1,$2,false)` instead of
   `SET name = value`, so a GUC value with spaces/`@`/`;`/`=` (an ec_distann roster
   spec) applies without a syntax error. Required for any roster-driven bench.
2. **Suite gate step** (fixture): builds `benchgate_corpus/_queries` from the
   clean corpus on each node, then runs `ecaz bench recall --profile ec_distann`
   against the coordinator single-node vs multi-node (roster session-GUC),
   comparing distinct_recall. Non-fatal (reports; the byte-identical top-k gate is
   the hard one).
3. **CustomScan iterative deepening** (`custom_scan.rs`): parity with the AM
   `amgettuple` path — re-run the search with a doubled exit bar on early-exit
   below top_k. Correct regardless (though it did not close the gap below).

## The finding (decision-grade)

```
RECALL_RESULT (default top_k)      : n_queries=50 identical=50 mismatched_ids=0
suite_recall_gate (ec_distann.top_k=32): single=0.5000 multi=0.3700 delta=-0.1300 pass=false
```

At the **default** top_k the multi-node top-10 is byte-identical to single-node
(packets 012/016). At **top_k=32** the multi-node path returns 10 results per
query but **worse** ones — a ~26% relative recall loss vs single-node. A direct
top-10 comparison at top_k=32 confirms it (5/50 queries identical, 228 mismatched
ids). This is a genuine multi-node recall regression at higher exploration, **not
tie-breaking noise** (too large), and the deepening fix did not close it — so the
divergence is in the multi-node orchestration/expander itself at larger top_k.

## Status

The byte-identical gate at default top_k stands, but is **insufficient across the
sweep**. This finding — multi-node recall trails single-node at `ec_distann.top_k`
≥ ~32 — is the top open issue and needs root-cause (likely the remote expander's
candidate set / exact-dist ordering diverging from the local expander under deeper
beam exploration). The suite-runner gate is now the tool to track it.

## Ask

Review the bench-runner GUC fix and the suite-gate wiring. **Prioritize
root-causing the multi-node recall gap at top_k=32** — it means the CustomScan
read path is not yet recall-equivalent to single-node across the tuning sweep.
