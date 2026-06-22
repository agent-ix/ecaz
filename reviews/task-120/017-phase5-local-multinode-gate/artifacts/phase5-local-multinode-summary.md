# Task 120 Phase 5 Local Multi-Node Summary

This packet is a local multi-node distributed SPIRE gate. It used one local
coordinator PostgreSQL instance and three local worker PostgreSQL instances on
the same physical machine. The workers had distinct SPIRE node identities
`2`, `3`, and `4`; this was not a single-node local scan.

AWS was not used. The local harness used direct local PostgreSQL sockets/ports
and generated remote shard TSVs under `target/`, outside the review packet.

## Topology

| Run | Coordinator | Worker nodes | Evidence |
| --- | --- | --- | --- |
| tiny smoke | port `39710` | ports `39711`, `39712`, `39713` | `static-remote-smoke/phase13e-static-remote-placement.log` |
| 10k | port `39720` | ports `39721`, `39722`, `39723` | `real10k-valid/smoke-customscan-read.log` |
| 50k | port `39730` | ports `39731`, `39732`, `39733` | `real50k/smoke-customscan-read.log` |
| 100k | port `39740` | ports `39741`, `39742`, `39743` | `real100k/smoke-customscan-read.log` |

All real-corpus runs published `1 128 3 published_static_remote_placements`,
planned `Custom Scan (EcSpireDistributedScan)`, and reported `remote_fanout: 3`.
The production-read smoke profile reported `local_pid_count=0`,
`remote_pid_count=24`, `result_source=remote_heap_candidates`,
`final_heap_fetch_status=remote_ready`, `status=ready`, and
`next_blocker=none`.

## Corpus And Shards

| Scale | Coordinator rows | Queries available | Query limit | Worker shard rows |
| --- | ---: | ---: | ---: | --- |
| 10k | 10,000 | 200 | 200 | node 2: 2,955; node 3: 3,639; node 4: 3,406 |
| 50k | 50,000 | 1,000 | 200 | node 2: 16,368; node 3: 15,845; node 4: 17,787 |
| 100k | 100,000 | 1,000 | 200 | node 2: 33,219; node 3: 29,471; node 4: 37,310 |

## Storage

| Scale | Total storage | Total indexes | SPIRE index | SPIRE bytes/row |
| --- | ---: | ---: | ---: | ---: |
| 10k | 168.4 MiB | 9.6 MiB | 9.4 MiB | 982.2 B |
| 50k | 835.6 MiB | 41.8 MiB | 40.7 MiB | 853.0 B |
| 100k | 1.6 GiB | 81.9 MiB | 79.7 MiB | 836.0 B |

## Recall And Latency

| Scale | Step | nprobe | recall@10 | p50 | p95 | p99 | Queries |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | default | 64 | 0.9850 | 42.148 ms | 49.005 ms | 57.794 ms | 200 |
| 10k | default | 96 | 0.9855 | 44.638 ms | 50.035 ms | 54.638 ms | 200 |
| 10k | rowcap25k | 96 | 0.9855 | 48.339 ms | 68.922 ms | 77.558 ms | 200 |
| 50k | default | 64 | 0.9850 | 54.194 ms | 62.366 ms | 71.122 ms | 200 |
| 50k | default | 96 | 0.9900 | 59.243 ms | 82.910 ms | 87.674 ms | 200 |
| 50k | rowcap25k | 96 | 0.9900 | 59.481 ms | 64.358 ms | 67.666 ms | 200 |
| 100k | default | 64 | 0.9730 | 78.257 ms | 117.404 ms | 125.179 ms | 200 |
| 100k | default | 96 | 0.9880 | 98.876 ms | 113.926 ms | 134.894 ms | 200 |
| 100k | rowcap25k | 96 | 0.9880 | 95.085 ms | 108.528 ms | 114.650 ms | 200 |

All production-read suite rows above reported `status=ready`,
`result_source=remote_heap_candidates`, `local_pid_sum=0`, `timeout_sum=0`,
`cancel_sum=0`, and `degraded_skip_sum=0`.

At these settings, `rowcap25k` did not bind: `selected_pid_sum` and
`remote_pid_sum` were `19200` for both nprobe `96` default and rowcap steps at
10k/50k/100k. The latency differences between the default and rowcap nprobe
`96` rows should not be interpreted as a proven rowcap win.

## Caveats

- The smoke handoff summary still reports `requires_remote_heap_resolution` for
  full-row handoff. The production-read profile and suite prove the compact
  remote heap-candidate production-read path with `id` projection, not final
  arbitrary full-row materialization.
- `ecaz bench spire-pipeline` emitted recall and latency metrics here, but did
  not emit NDCG.
- `ec_spire_index_placement_snapshot` was unavailable after remote placement
  publication because that helper currently expects local heap tuple delivery.
  Placement ownership is instead verified by the remote node snapshot plus empty
  `remote-leaf-materialization/*-missing-or-mismatched-leaves.txt` files.
