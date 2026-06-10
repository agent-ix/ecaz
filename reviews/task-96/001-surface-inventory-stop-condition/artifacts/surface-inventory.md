# Task 96 Phase 0 Surface Inventory

Task 96 requires an inventory before implementation:

- Audit current AMs for real TurboQuant no-QJL 2-bit scoring consumers.
- If no AM exposes such a surface, file the Stop Condition packet immediately
  and do not implement speculative kernels.

## Finding

No current AM exposes a real TurboQuant no-QJL 2-bit scoring surface.

The current codebase has 2-bit packing support in `ProdQuantizer`, but the
2-bit TurboQuant lane is QJL-enabled by construction, not no-QJL. The only
no-QJL LUT lane exposed to AMs is the canonical 4-bit tiled lane through
`PreparedLutNoQjl4BitQuery`.

## Source Evidence

### Quantizer-level shape

`src/quant/prod.rs` makes QJL inactive only when `bits == 4` and the dimension
has a supported tile:

```text
fn qjl_enabled(dim: usize, bits: u8) -> bool {
    !(bits == 4 && rotation::tile_dim(dim).is_some())
}
```

Therefore `bits == 2` always has `qjl_enabled(...) == true` in the current
implementation. `mse_bits(...)` then allocates `bits - 1` bits to MSE and the
payload includes a QJL residual-sign section.

The explicit no-QJL LUT preparation and scoring APIs are also hard-gated to
4-bit:

```text
prepare_ip_query_lut_no_qjl_4bit(...)
score_ip_from_parts_lut_no_qjl_4bit(...)
mse_code_bytes_no_qjl_4bit(...)
```

Each asserts `self.bits == 4 && !qjl_enabled(...)`.

### Type / SQL surface

`src/lib.rs` defines canonical TurboQuant datum defaults as:

```text
DEFAULT_QUANT_BITS = 4
DEFAULT_QUANT_SEED = 42
```

`validate_tqvector_bits(...)` rejects non-default `tqvector` bits, and
`encode_to_ecvector(...)` also requires canonical `(4, 42)` for the quantizer
path. This means the SQL-visible TurboQuant datum/index path is not exposing a
2-bit no-QJL TurboQuant scorer.

### AM consumers

The AM audit found only these TurboQuant no-QJL batch consumers:

- SPIRE: stores `no_qjl_4bit_lut: Option<PreparedLutNoQjl4BitQuery>` and calls
  `score_turboquant_no_qjl_4bit_batch_for(...)` only when that option is set.
- IVF: `IvfPreparedQuery::TurboQuantNoQjl4BitLut(...)` and
  `score_turboquant_no_qjl_4bit_batch_from_payloads(...)` are tied to
  `crate::DEFAULT_QUANT_BITS`.
- DiskANN: `DiskannTurboQuantPrefilterCodec` uses
  `PreparedLutNoQjl4BitQuery`, and its helper is named
  `turboquant_no_qjl_4bit_quantizer`.
- HNSW: exact modes named `FullLut`, `TiledLut`, and `Int8Approx` all route
  through no-QJL 4-bit query types; the batch helper is
  `score_turboquant_no_qjl_4bit_batch_for(...)`.

No AM currently has a `PreparedLutNoQjl2BitQuery`, `score_*no_qjl_2bit*`, or
equivalent 2-bit no-QJL batch route.

## Decision

Trigger the Task 96 stop condition. Do not land a speculative 2-bit no-QJL
kernel family until a real AM/storage surface exists.

The likely follow-up is a storage/scoring-surface task that first decides
whether 2-bit TurboQuant should be:

- a QJL-enabled lane, which belongs to Task 97-style gamma/residual-sign block
  kernels; or
- a new no-QJL 2-bit surface with explicit production consumers, in which case
  Task 96 can resume with a `lut32_2bit` or parameterized LUT kernel.

## Validation

No runtime tests or benchmarks were run. This packet is a static Phase 0 source
inventory, and the task file explicitly says to stop before speculative kernel
implementation when no consumer exists.
