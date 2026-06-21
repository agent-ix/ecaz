# Task 111h / Packet 024 Review Request: 100k v7 Rerank Format Sweep

## Summary

This packet requests review for a Task 111h benchmark evidence slice after the
RaBitQ residual sidecar fix in packet 022 and the centroid-relative TurboQuant
fix in packet 023.

The packet runs the 100k real-corpus rerank format/width sweep through
`ecaz bench suite` for:

- source f32 baseline,
- persisted index f16,
- persisted index RaBitQ4,
- persisted index RaBitQ8,
- persisted index TurboQuant.

The key artifact is `artifacts/summary-nprobe32.md`, backed by the packet-local
JSONL report files listed in `artifacts/manifest.md`.

## Execution Notes

The initial all-in-one suite hit ENOSPC after source f32, f16, and RaBitQ4
width 32. The packet records that failure and its database-size context:

- `artifacts/suite-status-after-enospc.log`
- `artifacts/db-sizes-after-enospc.log`
- `artifacts/postgres-largest-relations-after-enospc.log`

I then recreated the scratch database and ran clean continuation suites for:

- RaBitQ4 widths 64, 128, and 256,
- RaBitQ8 widths 32, 64, 128, and 256,
- TurboQuant widths 32, 64, 128, and 256.

Each continuation has a suite config, suite manifest, status log, report log,
and report JSONL under this packet. Truth caches were generated but intentionally
left uncommitted per repository packet rules.

## Key Results

At nprobe=32:

- source f32: recall@10 `0.9285-0.9350`, p50 `4.50-13.7 ms`, ec_ivf index `24.6 MiB`
- index f16: recall@10 `0.9280-0.9345`, p50 `4.36-14.3 ms`, ec_ivf index `323.7-342.0 MiB`
- index RaBitQ4: recall@10 `0.8910-0.8945`, p50 `3.81-6.67 ms`, ec_ivf index `104.0-121.8 MiB`
- index RaBitQ8: recall@10 `0.9010-0.9060`, p50 `3.80-8.62 ms`, ec_ivf index `177.4-195.4 MiB`
- index TurboQuant: recall@10 `0.9040-0.9075`, p50 `3.91-6.74 ms`, ec_ivf index `101.8-121.8 MiB`

At nprobe=200:

- source f32 reaches recall@10 `0.9875-0.9990`.
- index f16 reaches recall@10 `0.9870-0.9980`.
- RaBitQ4 tops out at recall@10 `0.9330-0.9380`.
- RaBitQ8 tops out at recall@10 `0.9455-0.9525`.
- TurboQuant tops out at recall@10 `0.9525-0.9565`.

## Interpretation

This packet does not support the earlier loose claim that f16 is a 150 ms path:
in this 100k v7 slice, f16 nprobe=32 p50 is `4.36-14.3 ms` and p99 is
`7.90-49.3 ms`. That said, the current persisted f16 index layout is still not
a win: it preserves source-like recall, but it makes the ec_ivf index roughly
`323.7-342.0 MiB` versus the source f32 baseline's `24.6 MiB`, and tails widen
at larger rerank widths.

RaBitQ8 and TurboQuant improve on RaBitQ4 recall, but neither matches source f32
or f16 recall in this 100k sweep. TurboQuant is the strongest compact quantized
format in this packet: it keeps RaBitQ4-like index size while slightly beating
RaBitQ8's high-nprobe recall ceiling.

## Review Ask

Please review:

- whether the packet-local artifacts support the interpretation above,
- whether the ENOSPC/continuation methodology is acceptable for a 100k evidence
  slice,
- whether any result should be excluded because it came from the interrupted
  main suite rather than a clean continuation,
- whether the next Task 111h slice should prioritize table-owned persisted
  storage, legacy `0x2A` attribution, read-amplification/stage counters, or
  the remaining 10k/50k/1M matrix.

## Non-Claims

This packet is not a Task 111h closeout. It does not claim completion of:

- the full 10k/50k/100k/1M matrix,
- table-owned persisted compact payload storage,
- legacy `0x2A` direct-TID sidecar benchmarking,
- cold-cache or remote-storage validation,
- payload byte/page-read and decode/scoring stage counter coverage,
- the final promote/iterate/abandon decision table.
