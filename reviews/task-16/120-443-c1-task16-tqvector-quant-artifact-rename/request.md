# Review Request: C1 Task16 Tqvector Quant Artifact Rename

Current head at execution: `8e2add6`

## Context

Packet `442` landed the important architectural change:

- `ecvector(dim)` became the canonical raw row type
- the quantized row surface stopped being the default indexed-column model

After that landed, the remaining naming problem became obvious:

- `ecvector` is product-correct for the canonical exact/raw type
- but the sibling quantized artifact type was temporarily named
  `ecqvector`, which is too generic
- the persisted artifact is specifically the current TurboQuant-family
  quantized payload

This slice fixes that naming mistake without reverting the row model.

## What Landed

### Canonical model stays the same

- `ecvector(dim)` is still the canonical raw row type
- the default indexed-column path still uses `ecvector`
- build / insert / scan / vacuum still fall back to the indexed `ecvector`
  column by default

This slice does **not** restore the old “quantized row as the primary
column” design.

### Quantized sibling renamed back to `tqvector`

- `sql/bootstrap.sql`
  - public quantized type is now `tqvector`
  - encoder is now `encode_to_tqvector(...)`
  - opclass is now `tqvector_ip_ops`
  - operator/helper names now consistently use `tqvector_*`
- `src/am/source.rs`
  - indexed quantized kind is now `IndexedVectorKind::Tqvector`
- `src/am/build.rs`, `src/am/scan.rs`, `src/am/vacuum.rs`, `src/am/insert.rs`
  - runtime messages and resolution paths now describe the quantized sibling
    as `tqvector`
- `src/lib.rs`
  - explicit quantized-artifact pg tests now create and query `tqvector`
    columns and use `tqvector_ip_ops`

## Validation

Green checkpoint on current head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Notable surfaces exercised by that checkpoint:

- `tqvector` text/binary I/O and encode path
- indexed `tqvector` explicit quantized-artifact tests
- indexed `ecvector` canonical raw-column tests
- PqFastScan/TurboQuant rerank-resolution tests that distinguish
  indexed `ecvector` from indexed `tqvector`

## Readout

### 1. The type taxonomy is now coherent

- `ecvector(dim)` = exact/raw canonical row type
- `tqvector` = TurboQuant-family persisted quant artifact

That matches the intended rule that family-specific persisted quantized
artifacts should use family-specific type names.

### 2. The row-model correction from packet `442` remains intact

This slice is naming-only at the product surface. It does not move the
canonical row back to a quantized type, and it does not reintroduce the
old duplicated-row assumption as the default model.

### 3. Future quant families can follow the same sibling-type pattern

Current head now has the intended separation:

- one canonical raw type (`ecvector`)
- one family-specific quantized sibling (`tqvector`)

If future persisted quantized families are added, they should follow the
same pattern rather than reusing a generic quantized type name.

## Review focus

1. Is the public SQL surface now consistently `tqvector` with no accidental
   leftover `ecqvector` names in runtime/user-facing paths?
2. Do the indexed-column resolution paths still clearly distinguish
   canonical `ecvector` from artifact `tqvector` across build and scan?
3. Is there any place where the rename accidentally suggests `tqvector`
   is the canonical row type again, rather than the explicit artifact type?
