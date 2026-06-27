---
id: FR-001
title: tqvector Data Type Registration
type: FR
status: APPROVED
object: entity
traces:
  - US-001
  - US-004
---
# FR-001: tqvector Data Type Registration

## Description

The extension SHALL register a PostgreSQL data type named `tqvector` with the following properties:

| Property | Value |
|---|---|
| Type name | `tqvector` |
| Storage | `EXTERNAL` (variable-length, TOASTable) |
| typlen | `-1` (varlena) |
| Input function | `tqvector_in` |
| Output function | `tqvector_out` |
| Send function | `tqvector_send` |
| Receive function | `tqvector_recv` |

## Internal Binary Layout

Little-endian, packed:

| Offset | Size (bytes) | Field | Type | Description |
|---|---|---|---|---|
| 0 | 2 | dim | u16 | Vector dimensionality |
| 2 | 1 | bits | u8 | Quantization bits (2–8) |
| 3 | 8 | seed | u64 | Quantizer seed |
| 11 | 4 | gamma | f32 | Residual norm used by the QJL correction term |
| 15 | variable | code_bytes | [u8] | Bit-packed MSE indices followed by bit-packed QJL signs |

Definitions:
- Datum prefix length = `2 + 1 + 8 = 11` bytes (`dim`, `bits`, `seed`)
- Quantized payload length = `4 + ceil(dim * (bits-1) / 8) + ceil(dim / 8)` bytes (`gamma` + `code_bytes`)
- Code-bytes length = `ceil(dim * (bits-1) / 8) + ceil(dim / 8)` bytes (`mse_packed` + `qjl_packed`)

The persisted representation stores:
- `dim` MSE centroid indices at `bits - 1` bits each
- `dim` QJL sign bits
- one `gamma` scalar

The implementation MAY use an internal transform workspace whose dimension is `next_power_of_two(dim)`, but the persisted type SHALL store only the first `dim` MSE coordinates, the first `dim` QJL signs, and `gamma`.

## Properties

The packed `tqvector` datum (`pack`/`unpack` in `src/lib.rs`; `EncodedTq` in `src/quant/prod.rs`) carries the following fields. The on-disk wire form is `dim` (`HEADER_BYTES = 2`) followed by `gamma` (4 bytes) followed by the code bytes. `bits` and `seed` are NOT serialized — they are re-derived from the canonical defaults on `unpack`.

| Field | Type | Description |
|---|---|---|
| dim | u16 (2 bytes, little-endian) | Vector dimensionality; first field of the packed datum (`HEADER_BYTES = 2`). |
| gamma | f32 (4 bytes, little-endian) | Residual L2 norm of the MSE-decode residual; used by the QJL correction term. |
| mse_packed | `[u8]`, `ceil(dim * mse_bits / 8)` bytes | Bit-packed MSE centroid indices, where `mse_bits = bits - 1` when QJL is active and `bits` otherwise. |
| qjl_packed | `[u8]`, `ceil(dim / 8)` bytes (empty when QJL inactive) | Bit-packed QJL sign bits; concatenated after `mse_packed` to form the code-bytes section. |
| bits | u8 (not persisted on wire) | Quantization bits; pinned to the canonical default `DEFAULT_QUANT_BITS = 4` and re-derived on `unpack`. |
| seed | u64 (not persisted on wire) | Quantizer seed; pinned to the canonical default `DEFAULT_QUANT_SEED = 42` and re-derived on `unpack`. |

For a 1536-dim, 4-bit datum (QJL active, `mse_bits = 3`) the packed wire size is `2 (dim) + 4 (gamma) + 576 (mse_packed) + 192 (qjl_packed) = 774` bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | `tqvector` is visible in `pg_type` after `CREATE EXTENSION ecaz` | Test |
| FR-001-AC-2 | `tqvector` values are TOASTable; a 1536-dim 4-bit datum occupies 783 bytes total | Test |
| FR-001-AC-3 | Pack/unpack of `(dim, bits, seed, gamma, code_bytes)` round-trips losslessly for all valid parameter combinations | Test |

### FR-001-AC-1: Type exists after CREATE EXTENSION
After `CREATE EXTENSION ecaz`, the type `tqvector` SHALL be visible in `pg_type`.

### FR-001-AC-2: Varlena storage
Values stored in `tqvector` columns SHALL be TOASTable. A 1536-dim, 4-bit datum SHALL occupy `11 + 4 + 576 + 192 = 783` bytes total: 11-byte datum prefix, 772-byte quantized payload, and 768-byte `code_bytes` section.

### FR-001-AC-3: Binary layout correctness
Pack/unpack of `(dim, bits, seed, gamma, code_bytes)` SHALL round-trip losslessly for all valid parameter combinations.

## Dependencies

- **Upstream**: US-001, US-004 (traces)
- **Downstream**: FR-002, FR-003 (tqvector text and binary I/O trace this type)
