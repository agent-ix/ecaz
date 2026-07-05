# Artifact Manifest

- head SHA: `a3b21b6327c7e0e7c0371defd45c7fbdde4c740f`
- task bucket: `reviews/task-91/021-quantcodec-closeout`
- timestamp: `2026-06-09T06:45:13-07:00`
- lane / fixture / storage format / rerank mode: local source/status closeout audit; no SQL fixture; all Task 91 AM x quant evidence aggregated from reviewed packets
- table surface: not applicable; source/status audit only

## Artifacts

### `aggregate-parity-table.md`

- purpose: maps Task 91 AM x quant cells and acceptance criteria to reviewed
  packet evidence.

### `quantcodec-impl-audit.log`

- command:

```bash
rg "impl.*QuantCodec|impl QuantCodec|QuantCodec for" src/am -n
```

- key result: expected `QuantCodec` implementations for IVF, SPIRE, HNSW, and
  DiskANN.

### `task-status-audit.log`

- command:

```bash
rg "Status:|superseded|closed by reference|closeout|Task 90|Task 91" plan/tasks/90-diskann-turboquant-search-codec.md plan/tasks/91-cross-am-quantcodec-migration.md plan/tasks/README.md
```

- key result: Task 91 complete; Task 90 closed by reference.

### `adr-audit.log`

- command:

```bash
rg "status: ACCEPTED|ADR-071|ADR-072|QuantCodec|try_score_ip_candidate" spec/adr/ADR-071-unified-quantizer-interface.md spec/adr/ADR-072-index-local-quantized-codec-adapters.md spec/adr/index.md
```

- key result: ADR-071 and ADR-072 accepted and aligned with `QuantCodec`.

### `git-diff-check.log`

- command:

```bash
git diff --check
```

- key result: command exited 0 with no whitespace findings.

### `local-cargo-test-quant-codec.log`

- command:

```bash
cargo test -p ecaz --no-default-features --features pg18 quant_codec
```

- purpose: supplemental local-only closeout verification for the shared
  `QuantCodec` adapter surface after Task 91 was marked complete.
- key result: 22 tests passed, 0 failed. The filter covers IVF TurboQuant,
  IVF grouped-PQ, IVF RaBitQ, SPIRE TurboQuant/RaBitQ/PqFastScan handling,
  SPIRE selected-row and column batch helpers, cutoff routing through
  `QuantCodec`, DiskANN TurboQuant metadata, and LUT32 batch bit-exact parity.

## Supplemental Verification

The supplemental local test log was captured after the original closeout packet
was committed. No GitHub CI, AWS smoke tests, or AWS benchmarks were run.
