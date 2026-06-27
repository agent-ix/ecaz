# Apple Silicon (M5) Index × Quant × Option Matrix — Task 104 deliverable

This is the Task 104 scope-item-5 deliverable: the Apple-silicon
supported-target column for Task 99's project-level `(AM, quant, ISA)`
completeness matrix, alongside the Graviton 4 and AWS-Intel production
columns.

- Host: Apple M5 Pro, `aarch64-apple-darwin`, PostgreSQL 18.3 (Homebrew
  binaries, pgrx-managed cluster, port 28818), release backend
  (`ecaz_build_profile()` probed before every suite).
- Backend SHAs: packets 002–006 measured at dylib
  `11cc8654…` (head `16133580a`, pre block-kernel rewrite); packet 007
  re-measured the QJL lanes at dylib `a11db8fb…` (head `f88c640d3`,
  candidate-parallel qjl32 NEON kernel) and again at dylib `fda206be…`
  (head `5c44d9f45`, NEON octet remainder routing — the octet round is
  the citable QJL state).
- Fixtures: real dbpedia 10k @1536-dim (staged `task31_m5_dbpedia_staged`),
  synthetic isotropic 10k @1024-dim for the QJL lanes (packet 006
  fixtures, seeds 10401/10402). One index per table, `task104_*` prefixes.
- All runs suite-driven (FR-038); SuiteConfigs + manifests + per-cell logs
  live in packets `reviews/task-104/002…007`.
- Sweep axes: HNSW `ef_search` {32,80,200}; IVF/SPIRE `nprobe` {8,16,32}
  ({8,16} at 1024-dim); DiskANN `list_size` {64,128}; candidate-batch
  scoring on/off at every cell whose AM exposes the toggle
  (`ec_{hnsw,spire,diskann}.candidate_batch_scoring`, IVF via
  `--ivf-scratch-soa-batch-decode`); HNSW exact-score-mode sweep
  {full_lut, int8_approx, tiled_lut, exact}; IVF rerank {auto, off} and
  adaptive-nprobe; SPIRE leaf-block-rows variant; RaBitQ storage-bits
  sweep on IVF {1,2,4,8}.

## Scoring-share floor gate (kernel ns/candidate vs same-cell unbatched anchor)

Gate: kernel-on cell ≥1.5× the same-head non-batch anchor. Kernel rows
all report `isa=neon`, `scalar_candidates=0`, width histograms recorded
(IVF example: 1158 ge32 flushes vs 27 sub-32 at nprobe=8).

Attribution note (packet 007 review): "scalar_candidates=0" holds on the
batch-surface kernel rows. Kernel-on cells additionally record a separate
`isa=scalar` one-off row — the non-batch scoring path (rerank, entry
distances), identifiable by its empty width histogram — which doubles as
the same-cell floor-gate anchor. After the qjl32 NEON octet routing
(`5c44d9f45`) no family leaks batch remainder flushes to scalar on the
M5; the only by-design scalar batch work is sub-8 flushes (cascade-wide
convention).

| AM | quant lane | kernel ns/c (neon) | anchor ns/c | ratio | gate |
| --- | --- | ---: | ---: | ---: | --- |
| IVF | TQ no-QJL 4-bit (lut32, full-LUT) | 221.3–221.9 | 880.9–912.9 | ~4.1× | PASS |
| IVF | grouped-PQ (PqFastScan) | 30.4–30.9 | — (kernel-dominant, 980k cand) | ≫1.5× vs 891 ns/c TQ-class one-off | PASS |
| IVF | RaBitQ bits=1 | 63.9–64.1 | — (kernel-dominant) | ≫1.5× (Task 93 M5 anchors: NEON 2.7–5.8×) | PASS |
| IVF | TQ-QJL @1024 (octet round) | 167.9–168.6 | 585.1–602.3 | ~3.5× | PASS (was 0.83× pre-fix); residual scalar = one-offs + 9–10 sub-8 flushes (sub-8 stays scalar by cascade design across families) |
| HNSW | TQ full_lut (lut32 repack, Task 102) | 494.9 avg | 903.3 avg | **1.83×** | PASS — repack did NOT regress; no revert-to-v1 |
| HNSW | TQ int8_approx (int8_approx32) | 99–105 | — (kernel-dominant) | ≫1.5× (fastest HNSW exact mode on M5) | PASS |
| HNSW | TQ tiled_lut | — (NEON path is a scalar-delegating stub; 100% scalar at 1339.6 ns/c) | — | — | **retired confirmed** (47–48% slower than full_lut on Intel per Task 103; on M5 it has no kernel at all) |
| HNSW | RaBitQ (bits-1 sidecar) | ~65 | — (kernel-dominant) | ≫1.5× (Task 93) | PASS |
| HNSW | grouped-PQ | no batch engagement observed (counters zero with prefilter disabled) | — | — | **coverage gap recorded** — Task 94/101 sub-width backport lane, see notes |
| HNSW | TQ-QJL @1024 (octet round) | 168.3–168.9 | 581.5–584.8 | ~3.5× | PASS — after the NEON octet remainder routing (`5c44d9f45`) the batch surface carries 113k–183k candidates at `isa=neon` with zero batch-side scalar fallback; remaining scalar rows are the one-off path (empty width histograms) |
| SPIRE | TQ no-QJL 4-bit | 226.8 avg | 818.8 avg | 3.61× | PASS |
| SPIRE | TQ-QJL @1024 (octet round) | 168.6–169.2 | 595.0–602.2 | ~3.5× | PASS (was 0.83× pre-fix); batch-side scalar fallback eliminated by the octet routing (scalar 21k/43k → 5.3k/10.8k, one-off only) |
| SPIRE | RaBitQ | runs; counters not batch-attributed on this surface | — | — | measured e2e only |
| DiskANN | TQ no-QJL 4-bit | 298.7 avg | 891.1 avg | 2.98× | PASS |
| DiskANN | binary/Hamming sidecar (hamming32) | 7.1 | — (integer-exact popcount; Task 95 M5 closeout) | PASS (Task 95: e2e p50 −21%/−3%) | PASS |
| DiskANN | RaBitQ bits=1 | 65.6 | — (kernel-dominant) | ≫1.5× (Task 93) | PASS |
| DiskANN | grouped-PQ | scan routes through the binary sidecar prefilter (`quant=binary` counters); grouped-PQ batch arm engages via prefilter_kind=grouped_pq only | — | — | recorded |

## Recall (kernel-on vs kernel-off, every measured pair)

**40/40 on/off cell pairs byte-equal on `recall@k`** (see packet
results.jsonl files; comparison in packet 008 manifest). This includes the
QJL tolerance-lane cells pre- and post-kernel-rewrite — recall identical
at 0.1734/0.3141 (HNSW ef 32/80), 0.1609/0.2938 (IVF nprobe 8/16),
0.1609/0.2953 (SPIRE nprobe 8/16). Tolerance families per ADR-076: QJL
cells score within the 4-ulp/1e-6 pre-slice contract (unit gates in
packet 001); all other measured families are bit-exact contracts.

Absolute recall on the 1024-dim cells is low (≈0.17–0.31) because the
synthetic isotropic fixture is adversarial for quantized search; it is a
fixture property, identical across on/off/pre/post cells, and does not
affect parity or floor-gate conclusions.

## End-to-end p50 deltas (batch-on vs batch-off)

| Cell | sweep | p50 on (ms) | p50 off (ms) | delta |
| --- | --- | ---: | ---: | ---: |
| IVF TQ no-QJL | 8/16/32 | 0.46/0.72/1.15 | 0.99/1.77/3.08 | **−53.5% / −59.3% / −62.7%** |
| IVF TQ-QJL @1024 (octet round) | 8/16 | 0.35/0.53 | 0.67/1.18 | **−47.8% / −55.1%** |
| DiskANN TQ no-QJL | 64/128 | 0.68/0.95 | 1.05/1.54 | −35.2% / −38.3% |
| SPIRE TQ no-QJL | 8/16/32 | 4.59/7.97/14.10 | 5.08/9.10/16.00 | −9.6% / −12.4% / −11.9% |
| HNSW TQ int8_approx | 32/80/200 | 0.43/0.69/0.91 | 0.48/0.80/1.04 | −10.4% / −13.8% / −12.5% |
| HNSW TQ full_lut | 32/80/200 | 0.56/0.93/1.20 | 0.61/0.98/1.26 | −8.2% / −5.1% / −4.8% |
| HNSW RaBitQ | 32/80/200 | 0.43/0.78/1.00 | 0.46/0.87/1.06 | −6.5% / −10.3% / −5.7% |
| SPIRE TQ-QJL @1024 (octet round) | 8/16 | 3.72/6.58 | 3.94/7.26 | −5.6% / −9.4% |
| DiskANN grouped-PQ (sidecar-routed) | 64/128 | 0.47/0.59 | 0.48/0.64 | −2.1% / −7.8% |
| IVF grouped-PQ | 8/16/32 | 0.28/0.33/0.45 | 0.27/0.34/0.47 | ±4% (noise; kernel share small in e2e) |
| IVF RaBitQ bits=1 | 8/16/32 | 0.29/0.40/0.56 | 0.29/0.40/0.57 | ~0% (rerank-dominated e2e) |
| HNSW grouped-PQ | 32/80/200 | 0.49/0.86/1.55 | 0.48/0.86/1.55 | ~0% (no batch engagement, see gap) |
| HNSW TQ-QJL @1024 (octet round) | 32/80 | 1.09/2.18 | 1.32/2.52 | **−17.4% / −13.5%** (was neutral before the octet remainder routing) |

## kernel_status markers (absent / retired / no-kernel cells)

| Cell | marker | basis |
| --- | --- | --- |
| HNSW tiled_lut exact mode | `retired` (executed as confirmation) | Task 103 Intel retirement; on M5 the NEON path is a scalar stub at 1339.6 ns/c vs full_lut 495 ns/c kernel |
| HNSW RaBitQ bits=4 / bits=8 storage lanes | `structurally_absent` | ec_hnsw exposes no `quant_bits`; rabitq on HNSW is the single bits-1-sidecar lane (`encode_to_ecvector` accepts only canonical (4,42)) |
| SPIRE PqFastScan | `structurally_absent` | reloption parses but `encode_assignment_payload` unconditionally errors: "requires a persisted grouped-PQ model"; no fixture flow can build the index — **flagged to Task 99 as a product gap** (no end-to-end SPIRE PQ evidence exists on any host) |
| DiskANN TurboQuant @1024-dim | `structurally_absent` | ambuild: "TurboQuant storage_format requires a no-QJL 4-bit dimension lane" (1536-only), consistent with Task 96/97 surface inventory |
| IVF RaBitQ bits=2/4/8 storage lanes | measured (bits via `quant_bits` reloption; the rabitq32 kernel covers bits=1 only) | nprobe=16: bits=2 recall 0.9410 / p50 3.07 ms; bits=4 recall 0.9750 / p50 (packet 003 rb4 logs); bits=8 recall 0.9820 / p50 0.53 ms — packet 003 per-cell logs |
| f32 raw | documented no-kernel cell | canonical unquantized lane; no block kernel exists or is planned on any ISA |

## No-SVE dispatch-ladder validation (scope item 4)

- Every kernel-attributed counter row across all packets reports
  `isa=neon`; no `sve`/`sve2` row exists anywhere in the matrix.
- The `*_sve_*_when_available` unit hooks early-return on the M5
  (packet 001 logs) and `select_highest_isa` resolves Neon when
  sve/sve2 are absent (unit-gated in `src/quant/isa.rs`).

## macOS / pgrx environment deltas recorded for this lane

- Cluster: pgrx-managed data dir (`~/.pgrx/data-18`), Homebrew PG 18.3
  binaries, unix socket `/Users/peter/.pgrx`, port 28818 — not the
  Homebrew service postmaster; preflight restart must target
  `pg_ctl -D ~/.pgrx/data-18`.
- No core pinning is available on macOS; latency percentiles include
  P/E-core scheduling variance (default QoS). Benches ran with the
  machine otherwise idle; p95/p99 tails are wider than the
  Linux-pinned production lanes and should not be compared across OSes.
- `pg_test_*` live-PG tests stay excluded on macOS (known pgrx runtime
  blocker); multi-threaded `cargo test --lib` runs hit "postgres FFI may
  not be called from multiple threads" panics — single-thread is the
  reliable full-sweep mode on this host (packet 001).

## Kernel changes produced by this lane (criterion 5 — must land on main before the G4 trip)

1. `16133580a` — qjl32 NEON production-scorer alignment (+ stale test
   fixes). aarch64-only.
2. `f88c640d3` — candidate-parallel qjl32 NEON block kernel
   (667–684 → 167–185 ns/c, floor gate 0.83× → 3.2–3.5×). aarch64-only.
3. `5c44d9f45` — NEON octet entry + ISA-dispatched remainder routing for
   qjl32 (packet 007 review response; the remainder dispatch in
   `candidate_batch` is shared code but a pure rename on the x86 path).
4. `d1235077c` — `ecaz bench suite` runnable `retired` kernel_status
   (tooling).

The kernel changes are aarch64-only. Two shared surfaces were touched —
the `candidate_batch` remainder dispatch (a rename; the x86 octet path
still reaches the same AVX2 implementation) and the `ecaz bench suite`
runner (tooling, not scoring) — neither alters x86 scoring behavior, so
the Task 103 Intel cells do not require a re-run.
