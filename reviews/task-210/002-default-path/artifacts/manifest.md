# Task 210 P1 — default-path A/B manifest

## Provenance

- Task bucket/packet: `reviews/task-210/002-default-path/`
- Extension + CLI head: `9ffadf649` (`extension_git_sha` on every summary
  row), PG18 release build with `distann-head-attribution-benchmark`
- Suite config: `artifacts/task210-p1-default-path.json`
  SHA-256 `6e860cf000ef2ba2413805cc4e2bbd334e5961ea38f34d6fa5069d7404907e1e`
- Structured results: `artifacts/run/results.jsonl` (993 rows)
  SHA-256 `09398ee1c7a17829d55195901601ec287aebf9e3a79dd80f3fa704d8e7a2ca49`
- 6 steps, all succeeded, one isolated 3-node cluster per step (never a
  shared-table surface). BW=4, H=100, L=32, degree 32, head cap 4096, k=10,
  200 queries, 50 iterations, 10 warmups, sharded head off (pre-P2-promote
  default), gateway copies off. Run dirs under
  `~/.ecaz/clusters/task210-p1-*`, removed after capture.
- Corpora: `data/staged-current`, prefixes `ec_real_{10k,50k,100k}`; SHAs on
  every row in `results.jsonl`.

## Arms

| id | role | admissibility | what it is |
|---|---|---|---|
| `task210-p1-control` | control | conforming | shipped default on a cluster that never built a replica image |
| `task210-p1-candidate` | candidate | conforming | shipped default on a cluster **holding a Ready replica image**, `ec_distann.allow_nonconforming_replica` off |
| `task210-p1-replica-context` | context | nonconforming | explicit opt-in replica, run first on the candidate cluster: builds the Ready image and measures the accelerator honestly |

## The clause-4 property — the default does not silently use the replica

`replica_scans` (latency command, 50 scans, same cluster for context and
candidate):

| scale | replica-optin-context | candidate (image present, GUC off) | control (no image) |
|---|---:|---:|---:|
| 10k | 50 | **0** | 0 |
| 50k | 50 | **0** | 0 |
| 100k | 50 | **0** | 0 |

The context arm proves the image is real and usable — every one of its scans
was served by the replica. The candidate arm, on the same cluster with the
same Ready image, used it zero times. This is the fixture the handoff §5.2
required: presence of the non-conforming structure changes nothing without
the explicit opt-in.

NFR-021 verdict rows (`physical_benchmark_nfr_021_conformance`):
control and candidate `actual_admissibility=conforming`,
`decision_eligible=true`; replica context `nonconforming`,
`decision_eligible=false`, `max_unsharded_derived_bytes=1,659,518,976` — the
1.66 GB coordinator-resident copy, itemised rather than invisible. All three
match their pre-registrations. The known head gap
(`ec_distann_generation_head_sample`/`_head_graph`, task-210-P2 allowlist) is
carried on every row: these arms ran the pre-P2 coordinator-local head, and
the gap closes with the P2 promote, not this packet.

## Recall — identical wherever the default path runs

| scale | control | candidate | replica context |
|---|---:|---:|---:|
| 10k | 0.9990 | 0.9990 | 0.9990 |
| 50k | 0.9545 | 0.9545 | 0.9540 |
| 100k | 0.9275 | 0.9275 | 0.9280 |

Candidate recall is bit-identical to control at every scale — the strongest
evidence that the same code path executed, image or no image.

## Latency — the image's presence costs nothing on the path

Warm mean (p50):

| scale | control | candidate | replica context |
|---|---:|---:|---:|
| 10k | 29.50 (29.20) ms | 29.60 (29.80) ms | 31.90 (31.40) ms |
| 50k | 39.40 (38.50) ms | 44.60 (43.50) ms | 37.10 (36.10) ms |
| 100k | 38.90 (37.10) ms | 39.30 (37.00) ms | 35.10 (33.50) ms |

Candidate matches control within noise at 10k (+0.3%) and 100k (+1.0%). The
50k delta (+13% mean, +13% p50) is larger than the other scales; recall and
`replica_scans=0` rule out a path change, so this reads as run-to-run
cluster variance (single 50-iteration sample per arm), and it moves in the
direction opposite to what silent replica use would produce (the replica is
*faster* — see the context column). Recorded honestly, not explained away.

The context column also documents why the replica became the latency control
in Tasks 198/199: it wins at 50k/100k. That win is now labeled
`nonconforming, decision_eligible=false` instead of steering the program.

## Re-run

    ecaz bench suite run \
      --config reviews/task-210/002-default-path/artifacts/task210-p1-default-path.json \
      --artifact-dir reviews/task-210/002-default-path/artifacts/run
