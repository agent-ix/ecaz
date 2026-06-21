# Review Request: Task 111h TurboQuant Borrowed Rerank Payload Batch

This packet requests review for commit `93632b77d`, which removes the survivor
payload slab copy from the index-side TurboQuant compact rerank path.

Code changed:

- `src/am/ec_ivf/rerank.rs`
- `src/am/ec_ivf/scan.rs`
- `src/tests/ec_ivf.rs`

What changed:

- Added a borrowed sidecar payload batch scorer for TurboQuant rerank payloads.
- Routed `rerank_probe_candidates_index_side` through borrowed payload refs when
  the scorer supports it.
- Kept the existing contiguous slab fallback for RaBitQ formats.
- Extended the existing PG18 byte-counter fixture to build an index-side
  TurboQuant rerank index and assert:
  - no heap source-vector bytes are read during rerank,
  - compact payload bytes are scored,
  - `rerank_payload_slab_bytes_copied == 0`.

Validation:

- `cargo test --no-default-features --features pg18 index_side_quantized_payloads_require_centroid_and_apply_correction`
  - log: `artifacts/cargo-test-rerank-borrowed-tq.log`
  - result: `1 passed; 0 failed`
- `cargo pgrx test pg18 test_ec_ivf_index_placement_fewer_rerank_bytes`
  - final log: `artifacts/cargo-pgrx-test-pg18-turboquant-borrowed-final.log`
  - result: `1 passed; 0 failed`

Non-claim: this is a partial copy/slab cleanup, not a full Task 111h closeout.
RaBitQ-4 and RaBitQ-8 still use the contiguous slab path because that is the
current fast arithmetic batch estimator route. Remaining 111h closeout items
still include RaBitQ slab evidence or implementation, legacy `0x2A` evidence,
table-owned storage evidence/replacement rationale, cold/remote evidence, and
the final decision table.
