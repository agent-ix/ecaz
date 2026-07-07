# Task 168 Packet 004 — frontier/alloc cleanups + storage-format default flip (Phase 4)

- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md`; work branch
  `task-168-diskann-batched-beam`.
- Commits under test: `fa0861ddd` (pooled decode + buffer reuse + TidHasher),
  `4050ea830` (StorageFormat::DEFAULT PqFastScan → RaBitQ + pg_test pin),
  `87d106d52` (TidHasher finish avalanche — the decision arm's binary).
- Baseline: packet 002 `results-w4.jsonl` (binary `1737ad5be`, W=4 default
  fixture arms).
- Host / backend: Intel desktop, PG18 pgrx tree (port 28818), db
  `tqvector_bench`; release verified per install (`build-profile.log`).
- Fixture: packet-001 indexes reused (`t168_p1_real{10k,50k,100k}_diskann`,
  rabitq, W=4 beam default); truth caches from packet 001. The default-flip
  commit does not affect these indexes (built with explicit
  `storage_format=rabitq`); its behavioral surface is covered by
  `cargo pgrx test pg18 ec_diskann`.
- Arms (all `ecaz --host /home/peter/.pgrx --port 28818 bench suite run ...`):
  - `suite.json` → `results.jsonl`: first after-arm (pre-hashfix binary
    `fa0861ddd`+`4050ea830`), recall+latency+profile.
  - `suite-rewarm.json` → `results-rewarm.jsonl`: latency-only stability
    re-run, same binary.
  - `suite-hashfix.json` → `results-hashfix.jsonl`: latency-only on
    `87d106d52`.
  - `suite-final.json` → `results-final.jsonl`: **decision arm** — full
    recall+latency protocol (matches how the baseline was produced) on
    `87d106d52`.
- Bespoke SuiteConfig justification: commit-level A/B packet over the
  packet-001 fixture.

## Decision-arm results (mean warm latency, W=4; recall bit-identical)

| scale | L | baseline | final | delta |
|---|---|---|---|---|
| 10k | 64 | 3.27 ms | 3.23 ms | −1.2% |
| 10k | 200 | 3.88 ms | 3.72 ms | −4.1% |
| 10k | 400 | 4.55 ms | 4.34 ms | −4.6% |
| 10k | 800 | 5.85 ms | 5.44 ms | **−7.0%** |
| 50k | 64 | 3.86 ms | 4.03 ms | +4.4% |
| 50k | 128 | 4.42 ms | 4.62 ms | +4.5% |
| 50k | 200 | 5.10 ms | 5.03 ms | −1.4% |
| 50k | 400 | 6.77 ms | 6.34 ms | **−6.4%** |
| 50k | 800 | 10.1 ms | 9.34 ms | **−7.5%** |
| 100k | 64 | 4.04 ms | 3.99 ms | −1.2% |
| 100k | 400 | 8.15 ms | 7.74 ms | **−5.0%** |
| 100k | 800 | 12.3 ms | 12.2 ms | −0.8% |

(L=128/200 rows at 100k: +0.4%/−1.9%; full tables in `results-final.jsonl`.)
recall@10 equals the baseline at every cell in every arm.

## Findings

1. **Landed**: pooled `decode_into` (zero steady-state allocation in the
   beam loop; previously 3 allocs + 3 frees per node), reused
   `neighbor_scores` buffer, and the multiply-shift TidHasher — after the
   `87d106d52` avalanche fix. Wins concentrate at L≥400 (−4.6 to −7.5%);
   the two +4.5% cells at 50k low-L are within the observed run-to-run
   spread for identical binaries (50k L=64 ranged 3.79–4.28 ms across
   this packet's arms).
2. **Hasher lesson** (first arm regressed up to +29%): a bare Fibonacci
   multiply leaves the product's low bits unmixed and hashbrown indexes
   buckets with the low bits — TID patterns clustered probes. `h ^= h>>32`
   in `finish()` flipped the A/B.
3. **Measurement protocol lesson**: latency-only suite runs (no preceding
   recall step) systematically inflate the first sweep point (L=64) by
   10–18% at 50k/100k across three independent runs. A/B latency arms must
   replicate the baseline's step protocol; the decision arm does.
4. Heap bounding from the task file skipped with evidence:
   `candidate_heap_us` < 3% of the frontier residual at every scale
   (packet 001).
5. Default flip: `StorageFormat::DEFAULT` PqFastScan → RaBitQ landed with
   `cargo pgrx test pg18 ec_diskann` 212 passed; the two remaining
   failures are the GUC threading flake (passes single-threaded) and
   `diskann_turboquant_prepared_prefilter_batch_scores_and_records_counters`,
   which fails identically on unmodified main-derived src (reproduced in
   the task-161 worktree) — pre-existing, tracked for follow-up.
