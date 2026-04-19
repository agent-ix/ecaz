# Review Request: C1 Native Build Real-Corpus Gate

Current head at execution: `81e86c0`

## Context

Packet `446` covered the native BUILD replacement itself plus synthetic/oracle
validation. This follow-up adds the first real-corpus gate readout on the
existing `50k` source-backed TurboQuant surface.

The goal here was narrower than another code slice:

- run the real `50k` gate against the native BUILD branch
- confirm the main operating points still sit in the expected post-task16 range
- record one harness wrinkle that still affects the per-index `summary` helper

## What I ran

Targeted the explicit `~/.pgrx` cluster with wrapper flags / explicit socket
targeting, then ran the existing gate harness on the TurboQuant storage-format
surface:

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --port 28817 \
  gate \
  --prefix tqhnsw_real_50k \
  --storage-format turboquant \
  --queries-table tqhnsw_real_50k_queries_50
```

This writes:

- [tmp/real_corpus_runs/20260419T200355Z_gate_tqhnsw_real_50k_turboquant_tqhnsw_real_50k_queries_50.tsv](/home/peter/dev/tqvector/tmp/real_corpus_runs/20260419T200355Z_gate_tqhnsw_real_50k_turboquant_tqhnsw_real_50k_queries_50.tsv)

I also attempted explicit-index summary runs for:

- `tqhnsw_real_50k_turboquant_m8_idx`
- `tqhnsw_real_50k_turboquant_m16_idx`

Those currently fail through the SQL helper with a grouped heap-f32 rerank path
selection that does not match the requested TurboQuant lane. I am not treating
that as a native-BUILD blocker for this packet because the gate report itself
completed successfully on the correct TurboQuant surface.

## Real-corpus result

The gate report returned:

- `(m=8, ef_search=40)  recall@10 = 0.886`
- `(m=8, ef_search=128) recall@10 = 0.930, exact@10 = 0.890`
- `(m=8, ef_search=200) recall@10 = 0.930`
- `(m=16, ef_search=200) recall@10 = 0.964`

These numbers are in the healthy range for the established `50k` TurboQuant
surface and are strong evidence that the native BUILD path is not causing a
large real-corpus recall regression.

## Harness note

The explicit summary helper attempts ended with:

`tqhnsw grouped heap-f32 rerank requires build_source_column, rerank_source_column, or TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN...`

That suggests the summary SQL path is still resolving a grouped / heap-f32
rerank surface even when the requested index is the TurboQuant index. This
looks like harness/config resolution drift, not a graph-build failure.

I did not patch that helper in this checkpoint because the acceptance-critical
piece for ADR-042 was the real-corpus gate surface itself, which did run.

## Review focus

1. Is the successful TurboQuant gate readout enough to treat real-corpus recall
   as provisionally satisfied for the native BUILD replacement?
2. Should the summary-helper drift be treated as follow-up harness work, or do
   you want it folded into the native-build branch before closeout?
