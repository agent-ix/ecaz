# Artifact Manifest

Packet: `reviews/task-111h/032-turboquant-borrowed-rerank`

Task bucket: `reviews/task-111h`

Code commit under review:
`93632b77d2058b679b89b0403df429b17f45107a`

Created: `2026-06-20`

## Scope

This packet covers a narrow Task 111h copy/slab cleanup slice:

- add a borrowed sidecar payload batch scorer for index-side TurboQuant rerank,
- route the IVF scan through borrowed payload references when the scorer supports it,
- preserve the existing contiguous slab fallback for RaBitQ formats,
- extend PG18 counter coverage so index-side TurboQuant reports zero survivor
  payload slab bytes copied.

Storage format / rerank mode:

- `storage_format = 'coarse_rerank'`
- `rerank_placement = 'index'`
- `rerank_format = 'turboquant'`

## Artifacts

| Artifact | Description |
| --- | --- |
| `cargo-test-rerank-borrowed-tq.log` | Focused Rust unit validation for borrowed TurboQuant sidecar batch scoring vs the existing corrected score path. |
| `cargo-pgrx-test-pg18-turboquant-borrowed.log` | First focused PG18 counter fixture run before removing unrelated test-file formatting churn; passed. |
| `cargo-pgrx-test-pg18-turboquant-borrowed-final.log` | Final focused PG18 counter fixture run from the committed worktree; passed. |

## Commands

Unit scorer validation:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 index_side_quantized_payloads_require_centroid_and_apply_correction" reviews/task-111h/032-turboquant-borrowed-rerank/artifacts/cargo-test-rerank-borrowed-tq.log
```

Final PG18 counter validation:

```sh
script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement_fewer_rerank_bytes" reviews/task-111h/032-turboquant-borrowed-rerank/artifacts/cargo-pgrx-test-pg18-turboquant-borrowed-final.log
```

## Key Result Lines

```text
test am::ec_ivf::rerank::tests::index_side_quantized_payloads_require_centroid_and_apply_correction ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2206 filtered out; finished in 0.08s

test tests::pg_test_ec_ivf_index_placement_fewer_rerank_bytes ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2206 filtered out; finished in 60.92s
```

## Non-Claims

This does not close the whole Task 111h copy/slab checklist line. RaBitQ-4 and
RaBitQ-8 still use the contiguous payload slab because their current fast
bits=4/8 path is the dedicated arithmetic batch estimator over contiguous code
bytes. That remaining work needs either an equivalent borrowed fast path or a
packet-local benchmark/evidence decision.

This packet does not provide legacy `0x2A`, table-owned storage, cold/remote
benchmark, or final promote/iterate/abandon evidence.
