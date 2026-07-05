# Task 76 Intel-Local SPIRE Pareto Summary

Suite: `task76-intel-local-spire-pareto`
Head: `4d832cdd4533e59864311e6e8918ce43ef63fddf`
Result: 33 completed, 0 failed, 0 skipped.

## Recommendation

Do not change SPIRE defaults from this packet.

The 10k SPIRE points are strong, but 100k high-recall SPIRE is still much slower than IVF. The best comparable 100k point:

- SPIRE tg96/nprobe96: recall@10 0.9975, p50 146.693 ms, p95 175.128 ms.
- IVF nprobe96: recall@10 0.9980, p50 37.7 ms, p95 46.5 ms.

100k SPIRE also hits a candidate plateau after nprobe64:

- nprobe64: recall@10 0.9825, leaf routes 3,556, candidates 2,784,952.
- nprobe96: recall@10 0.9975, leaf routes 3,556, candidates 2,784,952.
- nprobe128: recall@10 1.0000, leaf routes 3,556, candidates 2,784,952.

That means the final recall gains are not coming from a lower-cost routing envelope; they come from spending more work over the same broad candidate surface.

## Scope Notes

- 10k and 100k real-corpus fixtures were measured.
- The canonical 1M TSV fixture was not present locally, so this packet does not promote a 1M-informed SPIRE default.
- The suite used local Intel PG18, single-node local scans, and task-local table/index prefixes.
