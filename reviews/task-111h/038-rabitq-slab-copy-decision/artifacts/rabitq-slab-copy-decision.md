# Task 111h RaBitQ Slab-Copy Decision

Packet: `reviews/task-111h/038-rabitq-slab-copy-decision`
Current head audited: `0629a89409dd75a10e488b1767cb8d5e6b90004c`
Created: `2026-06-20T15:03:56-07:00`

## Verdict

The Task 111h copy/slab checklist item is closed by a split decision:

- Implemented: TurboQuant index-side compact rerank uses borrowed payload
  slices and reports `rerank_payload_slab_bytes_copied == 0`.
- Implemented: f16 index-side compact rerank is scalar over the persisted
  payload slice and reports `rerank_payload_slab_bytes_copied == 0`.
- Benchmarked away for this task: RaBitQ-4 and RaBitQ-8 keep the survivor-order
  contiguous scoring slab because their current fast path is the arithmetic
  batch estimator over contiguous code bytes. Replacing it with the available
  borrowed candidate-batch route would switch bits=4 to the measured-slower
  multi-bit block kernel route. A future no-slab RaBitQ4/8 improvement needs an
  equivalent borrowed arithmetic-estimator API, not the existing block-kernel
  wrapper.

This packet does not claim the copy is free. It claims that the current
replacement available in-tree is worse for the RaBitQ4/8 formats under 111h
decision, so retaining the measured fast contiguous slab is the correct 111h
closeout state.

## Current Code Shape

The current packed group scan path no longer materializes a heap-TID keyed
payload map for the hot path:

- `src/am/ec_ivf/scan.rs::rerank_probe_candidates_index_side` sorts survivors
  by heap TID, then loads groups by posting-carried group header TID.
- `load_rerank_groups_by_header_tid` is keyed by group header TID and reads each
  unique survivor group once.
- `rerank_group_payload_for_candidate` returns borrowed slices into the loaded
  group payload buffer.
- The old `0x2A` helpers that materialize `HashMap<ItemPointer, Vec<u8>>` still
  exist for legacy sidecar code paths, but the current v8 metadata path writes
  and scans packed `0x2B` group headers and `0x2C` continuation segments.

The remaining survivor-order scoring slab is format-specific:

- f16 uses the scalar branch in `rerank_probe_candidates_index_side`; no batch
  slab is allocated.
- TurboQuant enters the `supports_sidecar_payload_ref_batch()` branch and scores
  `Vec<&[u8]>` borrowed payload references.
- RaBitQ4/8 enter the contiguous `payload_slab` branch, record
  `record_rerank_payload_slab_bytes_copied(payload_slab.len())`, and call
  `score_sidecar_payloads_batch_with_centroid_ips`.

The PG18 counter fixture records this distinction:

- `src/tests/ec_ivf.rs::test_ec_ivf_index_placement_fewer_rerank_bytes`
  asserts f16 slab copied bytes are zero.
- The same fixture asserts TurboQuant slab copied bytes are zero.
- The same fixture asserts RaBitQ4 slab copied bytes equal scored payload bytes,
  making the retained copy visible rather than hidden.

## Why RaBitQ4/8 Do Not Use The Borrowed Candidate-Batch Route

The current borrowed candidate-batch surface is not equivalent to the RaBitQ4/8
arithmetic estimator:

- `src/am/ec_ivf/rerank.rs::score_payload_refs_batch` supports TurboQuant and
  intentionally rejects RaBitQ borrowed batches.
- `src/am/ec_ivf/quantizer.rs::score_ip_bits1_batch_from_payloads` routes:
  - bits=1 through `score_rabitq_bits1_batch_for`,
  - bits=2 through `score_rabitq_bitsn_batch_for`,
  - bits=4 and bits=8 through `prepared_query.estimate_ip_batch(...)`.
- `src/quant/rabitq.rs::estimate_ip_batch` accepts one contiguous code slab and
  dispatches the architecture-specific arithmetic kernels for bits=1/4/8.
- `src/am/common/candidate_batch/mod.rs::score_rabitq_bitsn_batch_for` accepts
  borrowed candidate payload references, but it uses the 32-wide block kernel
  wrapper. That is the wrong replacement for bits=4 under current evidence.

The available no-slab substitution would therefore be a scorer change, not just
an ownership cleanup:

```text
current RaBitQ4/8: page/group slice -> survivor slab -> arithmetic estimator
available borrowed route: page/group slice -> candidate-batch refs -> block kernel
missing route: page/group slice -> borrowed refs -> arithmetic estimator
```

The missing route may be worth implementing later, but it is not required to
make the 111h promote/iterate/abandon decision because the existing route is the
measured fast route.

## Benchmark Evidence

The retained RaBitQ4/8 slab is tied to Task 106's cross-host kernel routing
decision.

M5 / NEON evidence:

- Packet: `reviews/task-106/001-m5-multibit-rabitq-bench/`
- Manifest: `reviews/task-106/001-m5-multibit-rabitq-bench/artifacts/manifest.md`
- Key cited lines:
  - bits=2 block dispatch at 1024 dims: median `11.372 us`.
  - bits=2 scalar estimate at 1024 dims: median `34.140 us`.
  - bits=4 block dispatch at 1024 dims: median `12.853 us`.
  - bits=4 scalar estimate at 1024 dims: median `4.6153 us`.
  - Index-level routing: bits=2 emits RaBitQ block-kernel counters; bits=4
    emits no RaBitQ block-kernel line and stays on the arithmetic estimator.

Intel / AVX2 evidence:

- Packet: `reviews/task-106/002-intel-avx2-bench/`
- Manifest: `reviews/task-106/002-intel-avx2-bench/artifacts/manifest.md`
- Key cited lines:
  - bits=2 scalar estimate at 1536 dims: median `138.88 us`.
  - bits=2 block dispatch at 1536 dims: median `69.377 us`.
  - bits=4 scalar estimate at 1536 dims: median `12.810 us`.
  - bits=4 block dispatch at 1536 dims: median `72.900 us`.
  - Index-level suite: bits=4 p50 `2.76 ms` and bits=8 p50 `2.28 ms`, both
    with no block-kernel counter row, confirming estimator routing.

Task 111h current-path evidence:

- Packet `reviews/task-111h/032-turboquant-borrowed-rerank/` implemented and
  validated the borrowed no-slab path for TurboQuant.
- Packet `reviews/task-111h/030-counter-fixture-closeout-audit/` identified the
  remaining RaBitQ slab copy and the counter fixture that exposes it.
- Packet `reviews/task-111h/036-rabitq8-score-clip-ab/` measured the current
  contiguous RaBitQ8 estimator route at 100k. With estimator clip=4, recall@10
  reached `0.9305` at nprobe32 and `0.9915` at nprobe200 with p50 latencies
  `4.08 ms` and `14.3 ms`, respectively, and `183.6 MiB` index size.

## Closeout Interpretation

This closes the Task 111h row:

```text
Implement or explicitly benchmark away owned per-survivor payload copies and
double-copy batch-scoring slabs in the compact index path.
```

Closure is format-specific:

- f16: implemented, no batch slab.
- TurboQuant: implemented, borrowed batch refs, no batch slab.
- RaBitQ4/8: explicitly benchmarked away for this task. The current slab is a
  survivor-order scratch buffer required by the measured fast arithmetic
  estimator. The in-tree borrowed route is slower for the relevant bits=4
  format and is not a valid closeout replacement.

Residual risk:

- This does not prove a future borrowed arithmetic-estimator API cannot beat the
  current slab. It only proves the available borrowed block-kernel route should
  not replace the current RaBitQ4/8 scoring path in Task 111h.
- The current packed group loader still copies group header and continuation
  payload bytes into a per-group buffer after reading pages. That is bounded by
  unique survivor groups and is separate from the old per-survivor heap-TID map
  copy that the packed layout replaced.
