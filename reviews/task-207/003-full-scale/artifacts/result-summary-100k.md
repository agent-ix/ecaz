# Task 207 100k persisted-head A/B result

The suite compared `build_shards=1` control with `build_shards=4` candidate
using the same PG18 physical three-owner fixture, corpus/query digest, and
persisted-head evaluation strategy. Both arms passed topology and serving
gates with 100,000 source rows and zero non-owned rows/orphans.

| arm | recall | p50 latency | physical generation storage |
| --- | ---: | ---: | ---: |
| control (`build_shards=1`) | 0.9182 | 344.0 ms | 2,496,659,456 bytes |
| candidate (`build_shards=4`) | 0.9124 | 337.0 ms | 2,497,167,360 bytes |

Candidate delta versus control: -0.0058 recall, -7.0 ms p50 latency, and
`+507,904` bytes of physical generation storage (+0.0203%).

Source: `run-100k-final/results.jsonl`; extension SHA
`4e1e889784a58a82c02d7df503468f95e7c733df`, release profile with
`distann-head-attribution-benchmark`.
