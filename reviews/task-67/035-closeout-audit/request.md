# Task 67 Review Request: closeout audit

## Summary

This packet audits Task 67 against the amended 2026-05-30 measured closeout scope in `plan/tasks/67-rabitq-intel-avx-optimization.md`.

Verdict from coder side: implementation and measurement work is complete under the amended task definition. Several late packets are still awaiting outside reviewer feedback, so this request is the handoff point rather than a self-closed review topic.

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| AVX-512 and AVX2 bits=1/4/8 kernels landed | `src/quant/rabitq.rs` current x86 kernels; packets 001-008 | Complete |
| Batched scoring path for bits=1/4/8 | packets 003, 005, 007; kernel bench packet 020 includes batch rows | Complete |
| Differential-test scaffold includes x86 kernels | packets 011/012; `cargo test -p ecaz --lib quant::rabitq` in packet 034 passed 46 tests including `task67_sum_query_dequant_for_test_scaffold_*` and `x86_sum_query_dequant_*` | Complete |
| Intel kernel benchmark targets | packet 020 accepted by reviewer; packet 023 covers the bits=8 LS path | Complete |
| bits=1 SQL headline evidence | packets 022/026 document the SQL/top-k frontier path; under amendment, `nprobe=64` is the recall-preserving operating point | Complete |
| bits=8 SQL headline evidence across `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4` | packet 027, accepted as evidence by reviewer; result is honest failure of original strict 4x SQL threshold and accepted by amendment as bottleneck evidence | Complete |
| bf16 decision | packet 029 shows bf16-on preserves recall but is slower at all tested nprobe values; gate stays off | Complete |
| AVX2 fallback coverage | packet 033 amendment requires differential-test coverage but not AVX2-only host benchmark evidence; packet 034 quant tests cover AVX2 entries when available | Complete under amendment |
| Recall preservation | packets 017, 022, 027, 029 report recall parity/no regression in measured SQL lanes | Complete |
| AC4 no regression in DiskANN/HNSW/IVF scan tests | packet 034: DiskANN scan 18 passed, HNSW scan 73 passed, IVF scan 23 passed after correcting a stale test fixture | Complete |
| Safety constraints | packets 011/012 addressed target-feature scaffold and safety review; no new unsafe was added in packets 027-035 except existing cloud/test/doc/support code paths | Complete |
| AWS state | `artifacts/preflight/cloud-status-closeout.log`: `state: paused`, `$0.00/hr running` | Complete |

## Key Numbers

### bits=8 SQL Headline

Packet 027 measured scalar vs auto for all four variants at nprobe 16/32/64. Gate result under the original strict threshold: not met. Maximum observed total-bound speedup was 1.09x. Packet 033 amends closeout to accept this as measured bottleneck evidence because sidecar scoring is about 1% of total wall time.

### bf16

Packet 029:

| nprobe | bf16 off p50 | bf16 on p50 |
| ---: | ---: | ---: |
| 16 | 2.02 ms | 2.25 ms |
| 32 | 3.32 ms | 3.58 ms |
| 64 | 5.52 ms | 6.45 ms |

Decision: keep `rabitq-bf16` disabled by default.

## Validation Logs

Packet 034 contains the local validation logs used by this closeout:

- `cargo test -p ecaz --lib quant::rabitq`: passed, 46 tests
- `cargo test -p ecaz --lib am::ec_diskann::scan::tests`: passed, 18 tests
- `cargo test -p ecaz --lib am::ec_hnsw::scan::tests`: passed, 73 tests
- `cargo test -p ecaz --lib am::ec_ivf::scan::tests`: passed after fixture correction, 23 tests

## Open Review State

The following late packets were created after the last visible reviewer pass and still need outside review disposition:

- 029 bf16 SQL decision
- 031 cloud install clean target
- 032 cloud install pre-git clean
- 033 measured closeout amendment
- 034 IVF adaptive test fixture
- 035 closeout audit

Per `AGENTS.md`, this packet does not close those review requests locally.
