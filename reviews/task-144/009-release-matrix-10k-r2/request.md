# Task 144 Packet 009: 10k Release Matrix R2

Request review for the first real run of the approved packet-008 release matrix, limited to the 10k slice. This is not Task 144 closeout; it is the 10k decision readout before spending the 50k/100k run.

What changed since packet 008:

- Ran the approved r2 suite 10k slice on the Task-143-fixed branch under release backend profile.
- Fixed `bench storage` replica reporting: `mean_replicas_per_vector` now divides leaf assignments by corpus rows, not SPIRE health `object_count`.
- Reran all 10k storage summaries with the corrected release CLI.
- Preserved and processed packet-008 reviewer feedback, including the remote nudge covering 142/143/145/146 scope.

Key 10k findings:

- `probe_distance_ratio=1.25` is still dead: nprobe 96 recall is 0.7635 single, 0.7680 closure_e010, 0.7795 closure_e025, 0.8010 closure_e050.
- Ratio 2.0 improves but still misses 0.99 recall at nprobe 96 for the main closure variants.
- Ratio 4.0/8.0 can recover recall, but nprobe 96 usually exceeds the <=5% candidate row-instance AC.
- The strongest 10k AC rows are closure-based:
  - `closure_e050_b8-ratio400 @ nprobe16`: recall 0.9900, candidate 2.57%, ready 1.94%, production p50 7.670 ms, recall-harness p50 15.350 ms.
  - `closure_e050_b8-fixed @ nprobe16`: recall 0.9915, candidate 2.96%, ready 2.25%, production p50 7.833 ms, recall-harness p50 16.286 ms.
  - `closure_e025_b8-adaptive @ nprobe32`: recall 0.9935, candidate 4.36%, ready 3.95%, production p50 7.332 ms, recall-harness p50 20.881 ms.

Storage after corrected replica denominator:

- `single`: 17.9 MiB index, 1.0000 mean replicas/vector.
- `fixed_b2`: 34.9 MiB index, 3.0000 mean replicas/vector.
- `closure_e010_b8`: 18.5 MiB index, 1.0549 mean replicas/vector.
- `closure_e025_b8`: 20.4 MiB index, 1.2593 mean replicas/vector.
- `closure_e050_b8`: 26.0 MiB index, 1.9064 mean replicas/vector.

Evidence:

- Manifest: `artifacts/manifest.md`
- Suite manifest: `artifacts/suite-manifest-10k-r2.json`
- Suite results: `artifacts/results-10k-r2.jsonl`
- Suite log: `artifacts/suite-run-10k-r2.log`
- Corrected storage logs: `artifacts/storage-10k-*.log`
- Per-cell containment/identity tails: `artifacts/stage-containment-10k-*.jsonl`, `artifacts/result-identity-10k-*.jsonl`
- Focused validation: `artifacts/cargo-test-ecaz-cli-storage-r2.log`

Next after review: run the approved 50k/100k matrix slices, then take the promote / iterate / ADR-051-060 escalate decision.
