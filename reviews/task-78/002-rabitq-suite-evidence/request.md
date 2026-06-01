# Task 78 Review: RaBitQ Suite Evidence

Please review the Task 78 suite evidence and closeout decision.

This packet tests the first RaBitQ-first P0 slice from packet `001-rabitq-candidate-cutoff` against the Task 78 matched-recall latency gate. The result is a shelve, not a performance acceptance: the slice is code-correct, but it does not reduce the RaBitQ scored candidate surface and does not hit the required `>=10%` p50 improvement.

## What Was Measured

- Parent RaBitQ baseline at `7a8388efdf9519801eb121017b51a082366d1359`.
- Current RaBitQ cutoff slice at `c5b37ce0c38d0f23292dfa2595549c2c88a821c4`.
- Current TurboQuant comparison at the same points.
- 100k real corpus, 200 query rows, `top_graph_search_list_size` / `nprobe` points `64`, `96`, and `128`.
- All matrix runs used `ecaz bench suite`.

## Required Evidence Pointers

- Manifest: `artifacts/manifest.md`
- Latency/recall summary: `artifacts/latency-recall-summary.json`
- Funnel/stage summary: `artifacts/funnel-attribution-summary.json`
- Suite manifests:
  - `artifacts/baseline/suite-manifest.json`
  - `artifacts/current/suite-manifest.json`
  - `artifacts/turboquant-current/suite-manifest.json`
- Suite status:
  - `artifacts/baseline/suite-status.log`
  - `artifacts/current/suite-status.log`
  - `artifacts/turboquant-current/suite-status.log`
- PG18 clippy from the code packet: `reviews/task-78/001-rabitq-candidate-cutoff/artifacts/cargo-clippy-pg18.log`

## Key Result

| lane | storage | nprobe | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| baseline | rabitq | 64 | 0.9825 | 41.597 ms | 45.084 ms | 50.998 ms |
| current | rabitq | 64 | 0.9825 | 41.757 ms | 52.954 ms | 62.954 ms |
| baseline | rabitq | 96 | 0.9975 | 60.881 ms | 70.157 ms | 74.160 ms |
| current | rabitq | 96 | 0.9975 | 60.256 ms | 73.437 ms | 95.535 ms |
| baseline | rabitq | 128 | 1.0000 | 73.774 ms | 82.751 ms | 91.681 ms |
| current | rabitq | 128 | 1.0000 | 74.951 ms | 88.697 ms | 101.919 ms |

The current slice does not clear the p50 gate:

- nprobe64: `-0.4%` p50, worse.
- nprobe96: `+1.0%` p50, better.
- nprobe128: `-1.6%` p50, worse.

Candidate funnel rows also show no reduction in candidate surface:

| lane | nprobe | candidates | retained | returned | score p50 | score share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 64 | 10,420,357 | 5,000 | 2,000 | 20.705 ms | 87.2% |
| current | 64 | 10,420,357 | 5,000 | 2,000 | 22.490 ms | 88.1% |
| baseline | 96 | 15,506,227 | 5,000 | 2,000 | 31.384 ms | 87.5% |
| current | 96 | 15,506,227 | 5,000 | 2,000 | 33.768 ms | 88.1% |
| baseline | 128 | 20,000,000 | 5,000 | 2,000 | 39.315 ms | 87.3% |
| current | 128 | 20,000,000 | 5,000 | 2,000 | 42.446 ms | 88.0% |

## TurboQuant Comparison

RaBitQ remains the primary/default target. Current RaBitQ is materially faster than current TurboQuant at identical recall rows:

| storage | nprobe | recall@10 | p50 |
| --- | ---: | ---: | ---: |
| rabitq | 64 | 0.9825 | 41.757 ms |
| turboquant | 64 | 0.9825 | 89.144 ms |
| rabitq | 96 | 0.9975 | 60.256 ms |
| turboquant | 96 | 0.9975 | 129.835 ms |
| rabitq | 128 | 1.0000 | 74.951 ms |
| turboquant | 128 | 1.0000 | 167.193 ms |

TurboQuant also reports `requires_rabitq_storage_format` for the tuple transport status, while RaBitQ reports `ready`.

## Decision

Task 78's first P0 slice is shelved with packet-local evidence. The latency problem is still dominated by candidate volume under RaBitQ: scoring remains about `87-88%` of measured candidate-path CPU time, and the matrix still scores `10.4M`, `15.5M`, and `20.0M` candidates to return only `2,000` rows over 200 queries.

The validated RaBitQ lane should remain the primary/default policy direction relative to TurboQuant, but this packet does not by itself change a product default. A narrower default-policy task should make that change after a real candidate-selection win lands.

No SPIRE recursion semantic change is made in this packet.
