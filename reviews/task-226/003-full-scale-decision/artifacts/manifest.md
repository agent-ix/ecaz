# Task 226 packet 003 artifact manifest

- Task bucket / packet: `reviews/task-226/003-full-scale-decision/`
- Code correction SHA: `c85196ce841c1cbcea187dbefb3c10430fb611be`
- Clean release execution head: `a1f1584966011ca7c16175fe91f8efc302c8cf25`
- Suite runner binary SHA: `b54f321a579ccdac1535aedc4e3387f78811b0af`
- Lane: three-owner physical PG18 release extension, fixed 4,096 persisted
  sharded head, L32, H100, RaBitQ, lazy-10, 200 held-out queries, top-k 10
- Matrix: fresh 10k and 50k fixtures in this packet plus the immutable 100k
  production result in `reviews/task-226/002-current-head-100k/`
- Isolation: one immutable generation per scale shared only by BW4 control and
  BW8 candidate; no post-lifecycle fixture reuse
- SuiteConfig SHA-256:
  `876b63c99280911f1d8e7cb837ed379eeacbf806056d18230c9bc0ca3b5a751a`
- Run directories: `/home/peter/.ecaz/clusters/task226-bw8-full-10k` and
  `/home/peter/.ecaz/clusters/task226-bw8-full-50k`; both are outside the repo
  and were stopped and removed after packet-local evidence capture
- Storage format / rerank: unchanged physical RaBitQ generation; no format or
  rerank-mode change

## Preregistered evidence

- `task226-bw8-full-10k-50k.json`: release production SuiteConfig for the two
  remaining scales.
- `suite-audit.log`: config audit passes with exactly two steps.
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: expanded commands
  prove fresh external run directories and a BW4/BW8-only beam-width delta.
- Packet 002 100k gate: recall 0.9285 to 0.9450, paired delta +0.0165 with
  95% CI +0.0080 to +0.0265; warm mean 16.4 to 16.2 ms; p95 19.0 to 19.8 ms;
  A/A byte-identical.

## Execution

Command:

```text
/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-226/003-full-scale-decision/artifacts/task226-bw8-full-10k-50k.json --manifest-output reviews/task-226/003-full-scale-decision/artifacts/run/suite-manifest.json --results-output reviews/task-226/003-full-scale-decision/artifacts/run/results.jsonl --log-file reviews/task-226/003-full-scale-decision/artifacts/suite.log
```

- `run/suite-manifest.json`: generated at Unix ms `1787563140105`; config SHA
  matches the preregistration; both selected steps succeeded with exit code 0.
  The manifest records runner head `060e6794276d8ee1f90d2d57a2e79fe1051d0906`;
  the unanimous release extension preflight records execution SHA
  `a1f1584966011ca7c16175fe91f8efc302c8cf25`.
- `run/results.jsonl`: normalized 10k/50k provenance, generation, recall,
  latency, paired recall, topology, storage, engagement, and conformance rows.
- `run/{10k,50k}/distann-multinode-summary.log`: direct per-scale result rows.
- `run/{10k,50k}/physical-*-{recall,latency}.log` and prediction JSON:
  direct per-arm evidence. The 10k prediction files are byte-identical at
  SHA-256
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.
- `run/{10k,50k}/physical-head-membership.json`: 4,096-entry persisted-head
  membership evidence for each scale's immutable generation.
- `decision-summary.md`: compact 10k/50k/100k gate arithmetic and disposition;
  the 100k rows trace to packet 002's immutable normalized results.

## Key result lines

- 10k: recall 0.9990 / 0.9990, paired delta and CI exactly zero; BW4/BW8
  mean 14.80/14.20 ms and p95 17.90/16.80 ms. Branch (a) passes.
- 50k: recall 0.9540 / 0.9690, paired delta +0.015000, 95% CI
  `[+0.006500, +0.026000]`; mean 16.90/16.80 ms and p95 19.40/20.20 ms.
  Branch (b) passes.
- 100k (packet 002): recall 0.9285 / 0.9450, paired delta +0.016500, 95% CI
  `[+0.008000, +0.026500]`; mean 16.40/16.20 ms and p95 19.00/19.80 ms.
  Branch (b) passes.
- Storage is arm-invariant: 242,958,336 bytes at 10k, 1,243,561,984 bytes at
  50k, and 2,498,281,472 bytes at 100k. Coordinator-resident unsharded bytes
  are zero.
- Published topology is exact at each scale, with zero non-owned rows and zero
  orphans. Each arm engages both remote owners and passes materialization
  probes.
- Tail caveat: p99 regresses 7.14% at 50k and 5.08% at 100k, so the candidate
  remains policy-review gated despite passing the registered mean/p95 rule.

Disposition: `USEFUL NON-DEFAULT CONFIGURATION — EVIDENCE REVIEW-OPEN`. Task
219's review-closed recall-equivalence policy retains BW4 as the shipped
interactive default; Task 226 does not reopen it. Both external fixtures were
removed after evidence capture.
