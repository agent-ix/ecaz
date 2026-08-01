# Task 210 round 2 — zero-byte membership head A/B manifest

## Provenance

- Task bucket/packet: `reviews/task-210/006-zero-byte-head/`
- Extension: `01674b2bb` (`extension_git_sha` on every summary row; includes
  `35c7f3c3b` membership-blob persistence and `4070ff6cb` derived shard
  ordinal), PG18 release build with `distann-head-attribution-benchmark`.
- Suite config: `artifacts/task210-r2-zero-byte-head.json`
  SHA-256 `fd4bd83f478ffab6e78f1b2a76d514e2112be5a4cc1c845e66a07a1a3a4aeab4`
- Structured results: `artifacts/run/results.jsonl` (886 rows)
  SHA-256 `0397b1b188102a1e0b81d3d98d20a89b516cf7c3329e8ea350a1185cdd8be408`
- 9 steps, all succeeded, one isolated 3-node cluster per step. BW=4, H=100,
  L=32, degree 32, head cap 4096, k=10, 200 queries, 50 iterations, 10
  warmups. Run dirs under `~/.ecaz/clusters/task210-r2-*`, removed after
  capture.
- One evaluation nuance, disclosed: the first evaluation of these exact step
  artifacts failed because the suite reader did not know the emitter's new
  `nfr_021_class=control` vocabulary and mis-filed the head-state row
  (32,968 bytes: digests + the bounded membership blob) as derived corpus
  state. `0e4e28641` taught the reader the class; the verdicts below come
  from a `--resume-from` re-evaluation over the **unchanged** step artifacts
  (all 9 steps `already succeeded in resume manifest`).

## Arms

| id | role | admissibility | what it is |
|---|---|---|---|
| `task210-r2-local-head-context` | context | nonconforming | legacy coordinator-local head via `--local-head` — the referent, honestly failing the closed ratchet |
| `task210-r2-candidate` | candidate | conforming | shipped default, no flags: membership-only head as a state-row blob |
| `task210-r2-replica2` | context | conforming | shipped default + `head_replica_count=2`, attested population, members-derived shard ordinal |

## The zero-byte gate — met

Conformance rows (`physical_benchmark_nfr_021_conformance`):

| arm | coordinator resident unsharded | outstanding gap | verdict |
|---|---:|---|---|
| local-head context | 25,894,415 | head_graph + head_sample, `unowned` | nonconforming (as pre-registered) |
| candidate (default) | **0** | **none** | conforming, decision-eligible |
| replica2 | **0** | **none** | conforming |

The coordinator's head persistence is now the state row alone
(`nfr_021_class=control`, 32,968 bytes: digests, counts, and the u32+u64-ids
membership blob — bounded by capacity `C` exactly like the roster). The
sample table holds zero rows; no graph rows exist. The
`NFR_021_KNOWN_DISTRIBUTION_GAPS` allowlist is deleted: the context arm's
gap string reads `unowned`, and any future reappearance of a non-zero
coordinator head relation hard-fails the suite.

## Recall and latency

Recall@10 / warm mean (p50) ms:

| scale | local-head referent | candidate (default) | replica2 |
|---|---|---|---|
| 10k | 0.9990 / 29.20 (29.10) | 0.9990 / 31.70 (31.10) | 0.9990 / 31.10 (31.10) |
| 50k | 0.9545 / 40.20 (38.90) | 0.9545 / 40.00 (38.80) | 0.9540 / 40.50 (40.00) |
| 100k | 0.9275 / 37.40 (35.50) | 0.9280 / 39.30 (38.50) | 0.9290 / 38.40 (37.50) |

Recall is identical-or-better at every scale. The sharded default costs
+8.6% at 10k and +5.1% at 100k against the non-conforming referent —
consistent with 003a's finding and still the price of the invariant
(003a reviewer question 2 remains the place that cost is being weighed).

## Replica serving under the derived shard ordinal

`head_replica_shards_served` 29 / 33 / 32 at 10k/50k/100k, fallbacks 0.
Two artifacts of the previous round disappeared with the ordinal fix
(`4070ff6cb`): the 100k replica recall wobble (0.9265 → 0.9290, now above
the referent) and the +25% replica latency outlier (48.30 → 38.40 ms mean,
parity with the candidate) — both consistent with the diagnosis that
replica-built shards previously had per-serving-node topology and unstable
cache identity.

## Re-run

    ecaz bench suite run \
      --config reviews/task-210/006-zero-byte-head/artifacts/task210-r2-zero-byte-head.json \
      --artifact-dir reviews/task-210/006-zero-byte-head/artifacts/run
