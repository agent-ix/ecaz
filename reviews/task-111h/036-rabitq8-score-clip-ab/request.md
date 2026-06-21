# Task 111h / Packet 036 Review Request: RaBitQ8 Score/Clip A/B

## Summary

This packet requests review for the RaBitQ8 follow-up benchmark requested after
packet 035 made the score and clip levers benchmarkable.

The matrix uses `ecaz bench suite` on the real 100k corpus for:

- `rabitq_rerank_least_squares=0|1`
- `rabitq_rerank_clip=2|3|4`
- `rerank_format=rabitq8`
- `rerank_width=64`
- recall/latency sweep `nprobe=8,16,32,64,128,200`

The compact result table is `artifacts/summary-score-clip-ab.md`; provenance is
in `artifacts/manifest.md`.

## Execution Notes

The initial all-in-one suite completed both clip=2 variants and then hit ENOSPC
at the estimator clip=3 load step. That failure is preserved in:

- `artifacts/suite-status-after-enospc.log`
- `artifacts/suite-report-after-enospc.log`
- `artifacts/results-report-after-enospc.jsonl`

I then dropped and recreated the scratch database for each remaining variant:

- estimator clip=3
- least-squares clip=3
- estimator clip=4
- least-squares clip=4

Each continuation has its own suite manifest, run log, status log, report log,
raw JSONL, and report JSONL under this packet. Truth caches were generated but
removed before staging per repository packet rules.

## Key Results

At nprobe=32:

- estimator clip=2: recall@10 `0.9060`, p50 `4.34 ms`
- least-squares clip=2: recall@10 `0.9050`, p50 `4.23 ms`
- estimator clip=3: recall@10 `0.9260`, p50 `4.56 ms`
- least-squares clip=3: recall@10 `0.9250`, p50 `4.13 ms`
- estimator clip=4: recall@10 `0.9305`, p50 `4.08 ms`
- least-squares clip=4: recall@10 `0.9305`, p50 `4.25 ms`

At nprobe=200:

- estimator clip=2: recall@10 `0.9525`, p50 `14.5 ms`
- least-squares clip=2: recall@10 `0.9510`, p50 `14.3 ms`
- estimator clip=3: recall@10 `0.9830`, p50 `15.1 ms`
- least-squares clip=3: recall@10 `0.9825`, p50 `14.3 ms`
- estimator clip=4: recall@10 `0.9915`, p50 `14.3 ms`
- least-squares clip=4: recall@10 `0.9920`, p50 `14.5 ms`

All six variants report the same ec_ivf index size: `183.6 MiB`.

## Interpretation

The old clip=2 evidence was misleadingly pessimistic for RaBitQ8 rerank. Raising
the clip from 2 to 3 moves nprobe200 recall from about `0.952` to about `0.983`;
raising it to 4 moves the ceiling to about `0.992`, with similar warm latency
and unchanged measured index size.

Least-squares scoring does not look like the main lever in this run. It did not
improve clip=2 or clip=3, and at clip=4 it only improved nprobe200 recall by
`0.0005`.

## Review Ask

Please review:

- whether the packet-local artifacts support the clip=4 interpretation,
- whether the ENOSPC/one-variant-continuation methodology is acceptable,
- whether any clip=2 result should be rerun in a clean continuation instead of
  using the completed initial-suite rows,
- whether the next Task 111h implementation should default RaBitQ8 rerank clip
  to 4, expose it only as an experimental override, or run one more scale/lane
  before deciding.

## Non-Claims

This packet is not a Task 111h closeout. It does not claim completion of:

- table-owned persisted compact payload storage,
- f16 storage redesign,
- RaBitQ4 or TurboQuant score/clip equivalents,
- cold-cache or remote-storage validation,
- the full 10k/50k/100k/1M decision matrix.
