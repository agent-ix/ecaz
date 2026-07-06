# Task 144 Packet 010: 50k Release Matrix R2

Request review for the 50k slice of the approved packet-008 release matrix.
This packet is a measurement checkpoint, not Task 144 closeout; the 100k slice
is still required before the final promote / iterate / escalate decision.

Scope covered:

- Ran the approved r2 suite 50k slice under release backend profile.
- Preserved packet-008 reviewer requirements: release profile rows,
  harness-labeled production p50 and `result_source`, recall p50, candidate and
  ready row-instance percentages, and corrected `mean_replicas_per_vector`.
- Kept evidence packet-local under `reviews/task-144/010-release-matrix-50k-r2/`.
- Rechecked feedback buckets for assigned tasks 142-146 before preparing this
  packet; only existing local feedback is in Task 144.

Key 50k findings:

- The 10k AC shape does not reproduce at 50k.
- No 50k row satisfies both `distinct_recall@10 >= 0.99` and
  `candidate_row_instances_percent <= 5`.
- Every row that reaches 0.99 recall requires nprobe 96.
- `fixed_b2-adaptive @ nprobe96` is the least expensive 0.99 recall point:
  recall 0.9900, candidate 35.6834%, ready 20.6858%, production p50 20.434 ms,
  recall-harness p50 412.930 ms.
- The best closure 0.25 row is `closure_e025_b8-adaptive @ nprobe96`: recall
  0.9905, candidate 58.8173%, ready 22.3965%, production p50 30.042 ms.
- Closure 0.50 reaches 0.99 recall, but with very high scan and storage cost:
  80.9118%-86.0754% candidate row instances and 5.9123 mean replicas/vector.

Storage after corrected replica denominator:

```text
variant              index_size  mean_replicas_per_vector
single               50.4 MiB    1.0000
fixed_b2             129.0 MiB   3.0000
closure_e010_b8      91.6 MiB    2.0447
closure_e025_b8      168.1 MiB   4.0042
closure_e050_b8      242.7 MiB   5.9123
```

Evidence:

- Manifest: `artifacts/manifest.md`
- Suite manifest: `artifacts/suite-manifest-50k-r2.json`
- Suite results: `artifacts/results-50k-r2.jsonl`
- Suite log: `artifacts/suite-run-50k-r2.log`
- Release precheck: `artifacts/precheck-release-profile.log`
- Storage logs: `artifacts/storage-50k-*.log`
- Per-cell pipeline logs: `artifacts/pipeline-50k-*.log`
- Per-cell containment/identity tails: `artifacts/stage-containment-50k-*.jsonl`,
  `artifacts/result-identity-50k-*.jsonl`

Readout: this slice argues against promoting closure pruning on the 50k data
alone. Continue to the required 100k slice before making the final Task 144
decision.
