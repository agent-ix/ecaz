# Review request — Task 165: fix CustomScan exploration bar (recall parity) + suite gate PASS

**Branch:** `task-165-ec-distann-m3`. Fixes the multi-node recall regression the
suite gate caught in packet 019, and the suite-driven recall gate now **passes**.

## Root cause + fix

The CustomScan used the plan **LIMIT** as the FR-081 search exploration bar,
while the AM `amgettuple` path explores to **`ec_distann.top_k`** (the D9 exit
bar / ef-search knob, `routine.rs:255`). So at `ec_distann.top_k=32` with
`LIMIT 10`, the CustomScan searched with ef=10 while single-node used ef=32 →
under-explored → ~26% relative recall loss.

Isolated with `ec_distann_debug_expand_search`: the orchestration is byte-
identical single vs multi at top_k=32 (171 hits, 0 diff) — the bug was purely in
the CustomScan's exploration bar. Fix (`custom_scan.rs`): explore to
`options::current_top_k().max(LIMIT)`, iteratively deepen on early-exit, truncate
output to LIMIT.

## Evidence (`artifacts/`, real 3× PG18)

```
suite_recall_gate single=0.5000 multi=0.5000 delta=0.0000 pass=true
RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0   (default top_k)
recovery ... recovered=true
```

The multi-node recall now equals single-node **exactly across the sweep** — the
suite-driven gate (006-P1 letter) passes, and the byte-identical top-k gate still
holds. 110 pg_tests pass; clippy clean.

## Note

"Always measure" earned its keep: the default-top_k byte-identical proof
(packets 012/016) was necessary but not sufficient; the suite gate at
`ec_distann.top_k=32` surfaced a real recall bug that only a sweep-aware
measurement could catch.

## Ask

Review the exploration-bar fix. The read-path recall gate is now closed across
the tuning sweep (byte-identical AND suite-driven forms).
