# Task 67 Review Request: 100k AWS scale benchmark

## Summary

This packet adds checked-in `ecaz bench suite` configs and AWS benchmark evidence for the Task 67 scale follow-up.

**Superseded status:** the 100k scalar-vs-auto comparison in this packet is not Task 67 closeout evidence. Reviewer feedback found that the cloud wrapper did not propagate `ECAZ_SIMD` into the remote CLI process that runs `bench sidecar-rerank`, so the scalar/auto labels here do not prove different scorer kernels. Packet 037 fixes the runner issue, and packet 038 reruns the corrected 100k comparison.

This packet remains useful for its staged configs, 1m attempt/blocker evidence, and the record of the invalidated 100k run. Use packet 038 for 100k scalar-vs-auto numbers.

## Scope

- 100k RaBitQ8 IVF sidecar, scalar SIMD mode:
  `artifacts/task67-rabitq8-100k-scalar-suite.json`
- 100k RaBitQ8 IVF sidecar, auto SIMD mode:
  `artifacts/task67-rabitq8-100k-auto-suite.json`
- 1m HNSW AWS context:
  `artifacts/task67-hnsw-1m-suite.json`
- 1m DiskANN AWS context:
  `artifacts/task67-diskann-1m-suite.json`
- Minimal 1m HNSW/DiskANN configs:
  `artifacts/task67-hnsw-1m-min-suite.json`,
  `artifacts/task67-diskann-1m-min-suite.json`

## Superseded 100k Results

The artifacts below are retained for provenance, but the scalar-vs-auto comparison is superseded by packet 038.

- 100k scalar artifacts:
  `artifacts/100k-scalar/results.jsonl`,
  `artifacts/100k-scalar/suite-manifest.json`,
  `artifacts/100k-scalar/suite-run.log`,
  `artifacts/100k-scalar/load-100k-rabitq8-headline-scalar.log`,
  `artifacts/100k-scalar/sidecar-100k-rabitq8-headline-scalar.log`
- 100k auto artifacts:
  `artifacts/100k-auto/results.jsonl`,
  `artifacts/100k-auto/suite-manifest.json`,
  `artifacts/100k-auto/suite-run.log`,
  `artifacts/100k-auto/load-100k-rabitq8-headline-auto.log`,
  `artifacts/100k-auto/sidecar-100k-rabitq8-headline-auto.log`
- Comparison table:
  `artifacts/100k-comparison.tsv`

Superseded 100k lines:

- Sidecar score p50: scalar 0.022-0.023 ms; auto 0.022-0.026 ms.
- Total bound p50: scalar 12.297-21.971 ms; auto 12.455-22.131 ms.
- Total speedup: 0.982-1.000x across the RaBitQ8 variants and nprobe sweep.
- Recall: 0.9470-0.9940 across the same sweep.

## 1m Status

The `1m` AWS profile could not be provisioned because Terraform hit the account VPC quota; see `artifacts/preflight/1m-vpc-quota-note.md`. Partial resources were cleaned up with `ecaz cloud down`; final status for `1m` is `down`.

I also attempted a fallback 1m HNSW run on the already-running `10k-intel` profile. The full HNSW suite reached remote `ecaz bench suite` execution but failed before producing usable suite result artifacts; the failure is captured in `artifacts/1m-hnsw/cloud-bench-1m-hnsw-on-10k-intel.log`. The minimal 1m HNSW and DiskANN configs dry-run successfully, but no 1m result is claimed here.

Final cloud state: `10k-intel` is paused and `1m` is down.
