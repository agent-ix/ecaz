# Task 67 Review Request: closeout audit refresh

## Summary

This packet refreshes packet 035's closeout audit after the subsequent Task 67 review and benchmark work.

Packet 035 was conditionally approved as a faithful handoff, with the remaining blocker identified as procedural review disposition for packets 029, 033, and 034. Those packets now have outside reviewer approval. Packet 036 then attempted scale follow-up work, but its 100k scalar-vs-auto comparison was rejected because the cloud runner did not propagate `ECAZ_SIMD` into the remote CLI process. Packet 037 fixes that runner issue, and packet 038 supplies the corrected 100k AWS Intel comparison.

This packet does not mark Task 67 closed by itself. Packets 037 and 038 still need outside reviewer disposition before the refreshed handoff can be treated as fully closed.

## Closeout State

Under the 2026-05-30 measured-closeout amendment in `plan/tasks/67-rabitq-intel-avx-optimization.md`, the evidence map is:

| Closeout item | Current evidence | State |
| --- | --- | --- |
| AVX-512 and AVX2 kernel implementation plus differential scaffold | Packets 001-012, 020, 023, and packet 035 audit mapping | Complete under amendment; previously audited in packet 035 |
| Kernel-layer speedups meet per-kernel targets on Intel | Packets 020 and 023, with reviewer feedback present | Complete under amendment |
| bits=1 SQL pre/post, recall, and bottleneck attribution | Packets 022 and 026; `nprobe=64` is the amended recall-preserving gate | Complete under amendment |
| bits=8 SQL pre/post, recall, and bottleneck attribution | Packet 027 documents the strict 4x SQL target was not met and attributes the residual bottleneck outside scoring | Complete under amendment |
| bf16 decision | Packet 029 reviewer-approved: bf16 preserves recall but is slower, feature remains off by default | Complete |
| No DiskANN/HNSW/IVF regression | Packet 034 reviewer-approved: DiskANN/HNSW/IVF scan evidence accepted after the IVF fixture correction | Complete |
| Cloud benchmark SIMD selector validity | Packet 037 fixes the cloud runner to export `ECAZ_SIMD` into the remote `ecaz bench suite` process | Pending reviewer disposition |
| Corrected 100k AWS Intel scale evidence | Packet 038 supersedes packet 036's invalid 100k comparison | Pending reviewer disposition |
| 1m HNSW/DiskANN scale evidence | Packet 036 attempted 1m setup and documents the VPC quota blocker; no 1m result is claimed | Not delivered; not part of the amended Task 67 closeout gate |

## Corrected 100k Result

Packet 038 is the scale evidence to use, not packet 036's original scalar-vs-auto table.

Key packet 038 lines:

- Host: AWS `10k-intel`, DB instance `m7i.2xlarge`, `x86_64`, Intel processor family.
- Sidecar score p50: scalar `0.107-0.111 ms`, auto `0.019-0.022 ms`.
- Sidecar score speedup: `4.864-5.842x`.
- Total bound p50: scalar `13.433-24.287 ms`, auto `11.167-19.136 ms`.
- Total bound speedup: `1.197-1.271x`.
- Recall@10 range: `0.9470-0.9940`.

Interpretation: the corrected selector shows real SIMD scorer acceleration at 100k, while the end-to-end SQL/sidecar path remains dominated by candidate SQL and sidecar I/O.

## 1m Status

No 1m HNSW or DiskANN benchmark result exists for Task 67.

Packet 036 staged HNSW and DiskANN 1m suite configs and attempted AWS execution. The dedicated `1m` profile failed during Terraform apply because the AWS account hit the VPC quota. A fallback HNSW attempt on `10k-intel` also failed and is recorded in packet-local logs. This packet treats those artifacts only as blocker evidence, not benchmark evidence.

## Open Review State

- Packet 037: open; needs reviewer confirmation that the cloud runner SIMD propagation fix is correct.
- Packet 038: open; needs reviewer confirmation that the corrected 100k scalar-vs-auto evidence supersedes packet 036.
- Packet 039: this refresh packet; needs reviewer confirmation that the handoff map is current and honest.

No code or test changes are included in this packet.
