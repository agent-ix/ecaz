# Task 207 50k result summary

The final persisted-head PG18 A/B completed on 2026-08-03 with extension SHA
`4e1e889784a58a82c02d7df503468f95e7c733df` and a unanimous release preflight.
Both three-owner fixtures passed topology, serving, and remote-owner checks
with 50,000 source rows, zero non-owned rows, and zero orphans. The external
control and candidate run directories were removed after capture.

| arm | recall | p50 latency | physical generation bytes |
| --- | ---: | ---: | ---: |
| control, `build_shards=1` | 0.8814 | 333.0 ms | 1,242,734,592 |
| candidate, `build_shards=4` | 0.8994 | 323.6 ms | 1,245,241,344 |

The candidate gained 0.0180 recall and reduced p50 latency by 9.4 ms, with a
2,506,752-byte (0.20%) physical-generation increase. The complete structured
source is `run-50k-final/results.jsonl`; per-arm recall, latency, prediction,
topology, and storage logs are under `run-50k-final/{control,candidate}/`.
The 100k persisted-head A/B remains open in this packet.
