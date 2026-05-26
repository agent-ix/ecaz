# Task 55 AWS DiskANN Low-Cost Config Audit

Purpose: establish a trustworthy `ec_diskann` baseline on low-cost AWS
Graviton before doing code optimization. The first target is the `10k` cloud
profile (`m8g.large` from the Terraform profile) rather than the larger
`10k-medium` profile.

## Scope

- access method: `ec_diskann`
- hardware lane: low-cost Graviton only; Intel is deferred
- datasets: DBpedia/OpenAI3 `ec_real_10k` and `ec_real_100k`
- suite runner: `ecaz bench suite`
- claim class: benchmark-packet evidence, not product benchmark claim

The suite intentionally starts with configuration and graph-shape evidence.
Prior AWS DiskANN results were very slow at 1M and weakly sensitive to
`list_size`; this packet is meant to distinguish real implementation cost from
bad config, planner path, cache state, memory pressure, or graph quality.

## Comparator Notes

Immediate AWS target is `m8g.large` via the `10k` profile: 2 vCPU, 8 GiB,
EBS-only, up to 12.5 Gbps network and up to 10 Gbps EBS bandwidth per the
AWS M8g instance table. That is intentionally cost-first and memory-tight.

Follow-up Graviton comparators, still before Intel:

- `m8g.xlarge` or `m8gd.xlarge` if 8 GiB memory pressure or EBS behavior
  explains the bad shape.
- `c8g.xlarge` or `c8gd.xlarge` if CPU dominates after graph/config checks;
  C8g halves the memory-per-vCPU ratio versus M8g and is useful for separating
  CPU traversal cost from cache residency.
- `m8gb.*` or `c8gb.*` only if EBS wait shows up as the bottleneck; these have
  higher EBS bandwidth than same-sized base M8g/C8g instances.

External comparators are directional only. ANN-Benchmarks tracks recall/QPS,
index size, and build time across algorithms, but the public leaderboard is not
PostgreSQL AM shaped. The DiskANN NeurIPS 2019 paper is the algorithmic
reference point for high-recall disk-backed graph search, while pgvector HNSW
is the nearest PostgreSQL-native production comparator.

Sources:

- https://aws.amazon.com/ec2/instance-types/m8g/
- https://aws.amazon.com/ec2/instance-types/c8g/
- https://ann-benchmarks.com/index.html
- https://papers.nips.cc/paper_files/paper/2019/hash/09853c7fb1d3f8ee67a61b6bf4a7f8e6-Abstract.html
- https://github.com/pgvector/pgvector

## Suite

Config: `suite.json`

Expected command shape:

```text
target/release/ecaz bench suite audit --config benchmarks/task55-aws-diskann-lowcost-config-audit/suite.json
target/release/ecaz bench suite run --dry-run --config benchmarks/task55-aws-diskann-lowcost-config-audit/suite.json --manifest-output benchmarks/task55-aws-diskann-lowcost-config-audit/artifacts/suite-dry-run-manifest.json
target/release/ecaz cloud up --profile 10k --git-ref <branch>
target/release/ecaz cloud install --profile 10k --git-ref <branch>
target/release/ecaz cloud bench --profile 10k --suite task55-aws-diskann-lowcost-config-audit --database postgres --config benchmarks/task55-aws-diskann-lowcost-config-audit/suite.json --ecaz-bin target/release/ecaz
target/release/ecaz cloud snapshot --profile 10k --description task55-aws-diskann-lowcost-config-audit
target/release/ecaz cloud down --profile 10k --yes
```

## Artifacts

Artifacts land under `artifacts/`:

- `suite-manifest.json`, `results.jsonl`, `suite-run.log`
- corpus fetch/prepare/load logs
- `precheck-*.log`
- `graph-*.log`
- `build-probe-*.log`
- recall, latency, storage, and explain logs

## Acceptance

- suite audit and dry-run pass locally
- AWS suite completes without stale or missing artifacts
- every result row records DiskANN reloptions/GUCs through graph or cost
  diagnostics
- the manifest states whether the prior bad DiskANN shape reproduced on
  low-cost Graviton

## Result Summary

The config audit did not find a planner/config failure. The 100k explain log
shows planner scan selection live for `ec_diskann`, effective session
`list_size=200`, `storage_format=pq_fastscan`, graph degree 32, and 100k live
tuples. Storage was `46.1 MiB` / `483.1 B` per row.

The prior bad latency shape reproduced on the low-cost Graviton lane: 100k SQL
latency stayed nearly flat across the sweep, with means of `61.9 ms`, `63.1 ms`,
`61.7 ms`, `62.9 ms`, and `64.8 ms` for `list_size` 64, 128, 200, 400, and
800. Recall was healthy and increased with `list_size`: `0.9165`, `0.9625`,
`0.9745`, `0.9855`, and `0.9865`.

That combination pointed at a fixed per-scan implementation cost rather than
bad graph quality or a disabled planner path. The follow-up optimized packet is
`benchmarks/task55-aws-diskann-lowcost-optimized/`.
