# Task 85 Product-Scale Closeout Manifest

- head SHA: `2d638e086303dbe4eb16fe6841e53f0f1df0cd25`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/007-product-scale-closeout/`
- lane: AWS 1M/q500 SPIRE product-scale closeout
- host profile: AWS `1m`
- storage format: RaBitQ SPIRE, retained block16 and Task 85 block8/block32 variants
- rerank mode: heap rerank width 25 unless cited comparator packet differs
- timestamp: 2026-06-07
- isolated/shared surface: isolated one-index-per-table Task 85 SPIRE surfaces; comparator packets retain their own recorded surfaces

## Commands

Wrapper status capture:

```text
target/debug/ecaz cloud status --profile 1m --database postgres
```

Direct EC2 verification:

```text
aws ec2 describe-instances --instance-ids i-06ace3e95ab942623 --query 'Reservations[].Instances[].{InstanceId:InstanceId,State:State.Name,PrivateIp:PrivateIpAddress,PublicIp:PublicIpAddress,Name:Tags[?Key==`Name`]|[0].Value}' --output table
```

No new benchmark suite was run for this packet. The closeout cites already
completed `ecaz bench suite` evidence from the owning Task 85 packets and
immutable comparator benchmark packets.

## Artifacts

- `cloud-status-final-paused-closeout.log`: local cloud wrapper status capture.
  It returned `state: unknown`, so it is retained as evidence of the wrapper
  result rather than as proof of a paused state.
- `aws-ec2-status-final-closeout.log`: direct AWS EC2 status for the known
  Task 85/1M database instance. Key result: instance
  `i-06ace3e95ab942623`, name `ecaz-cloud-1m-db`, state `stopped`,
  private IP `10.42.1.131`.

## Source Evidence

Task 85 SPIRE evidence:

- `reviews/task-85/001-handoff-product-baseline-suite/`: retained Task 79/81
  baseline and Task 83 controls. Retained bar: recall@10 `0.9832`,
  `candidate_sum=9,213,846`, p50 `246.397 ms`, p95 `304.476 ms`, p99
  `321.342 ms`.
- `reviews/task-85/003-joint-miss-latency-targeting/`: selected-leaf misses
  dominate the retained gap; selected-leaf contextual misses `81` vs routing
  misses `3`.
- `reviews/task-85/004-aws-1m-block8-geometry/`: block8 rejected. Same recall
  at global1152, lower candidates, but p50/p95/p99 worsened.
- `reviews/task-85/005-aws-1m-per-leaf-block-cap/`: per-leaf caps rejected.
  Recall collapses and latency worsens; global allocation implicitly prunes
  object reads.
- `reviews/task-85/006-aws-1m-block32-geometry/`: block32 provides a layout
  signal but no product Pareto point. Matched-row budget loses recall; same
  recall roughly doubles candidates for only tiny latency movement.

Comparator evidence:

- `benchmarks/task51-aws-ivf-rabitq-final-gate/`: ec IVF/RaBitQ 1M comparator.
  nprobe128 recall `0.9864`, p50 `34.6 ms`, p95 `41.5 ms`, p99 `48.0 ms`,
  index `298.0 MiB`.
- `benchmarks/task59-aws-diskann-final-graviton-suite/`: ec DiskANN 1M
  comparator. L800 recall `0.9825`, p50 `19.7 ms`, p95 `30.9 ms`, p99
  `35.6 ms`, index `455.1 MiB`.
- `benchmarks/comparators-50k-100k-1m/`: external 1M comparator context.
  pgvectorscale DiskANN sl400 recall `0.984`, p50 `19.5 ms`, p95 `32.0 ms`;
  vchord RaBitQ default recall `0.9995`, p50 `90.3 ms`, p95 `100.0 ms`.

## Closeout Result

Task 85 closes with no product-scale SPIRE Pareto point justified yet. No
defaults, product profiles, ADRs, or current benchmark promotions are made from
Task 85.
