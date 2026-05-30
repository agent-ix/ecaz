# Task 67 Review Request: corrected 100k SIMD benchmark

## Summary

This packet supersedes packet 036's 100k scalar-vs-auto comparison with a corrected run after packet 037 fixed `ecaz cloud bench --simd-mode` propagation.

Packet 036 did run on AWS, but `bench sidecar-rerank` scores in the remote CLI process and the old cloud wrapper only set `ECAZ_SIMD` for PostgreSQL. Packet 037 fixed that. This packet reruns the same 100k RaBitQ8 sidecar surface using fresh prefixes and the fixed wrapper.

## Results

- Corrected comparison table: `artifacts/100k-comparison.tsv`
- Scalar artifacts: `artifacts/100k-scalar/`
- Auto artifacts: `artifacts/100k-auto/`
- Manifest: `artifacts/manifest.md`

Key lines:

- Sidecar score p50: scalar `0.107-0.111 ms`, auto `0.019-0.022 ms`.
- Sidecar score speedup: `4.864-5.842x`.
- Total bound p50: scalar `13.433-24.287 ms`, auto `11.167-19.136 ms`.
- Total bound speedup: `1.197-1.271x`.
- Recall@10 range: `0.9470-0.9940`.

This confirms the SIMD selector is now real and still shows the same broader bottleneck attribution: the kernel improves substantially, but SQL/candidate/sidecar I/O dominates the end-to-end bound at 100k.

## Scope Notes

- The benchmark host is AWS `10k-intel`, DB instance `m7i.2xlarge`, `x86_64`, Intel processor family. Attestation is under `artifacts/preflight/`.
- The scalar and auto runs use isolated prefixes:
  `task67_r8head_100k_scalar_envfix` and `task67_r8head_100k_auto_envfix`.
- This packet does not claim 1m HNSW or DiskANN results. Packet 036 remains the evidence for the 1m VPC quota blocker.

## Validation

- JSON configs parsed with `jq empty`.
- Both suite configs dry-ran successfully before AWS execution.
- AWS final state: `10k-intel` paused.
