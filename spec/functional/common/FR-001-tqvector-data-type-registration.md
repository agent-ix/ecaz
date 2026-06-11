---
id: FR-001
title: tqvector Data Type Registration
type: functional-requirement
status: APPROVED
object_type: entity
traces:
  - US-001
  - US-004
---
# FR-001: tqvector Data Type Registration

## Requirement

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

## Acceptance Criteria

### FR-001-AC-1: Type exists after CREATE EXTENSION
After `CREATE EXTENSION ecaz`, the type `tqvector` SHALL be visible in `pg_type`.

### FR-001-AC-2: Varlena storage
Values stored in `tqvector` columns SHALL be TOASTable. A 1536-dim, 4-bit datum SHALL occupy `11 + 4 + 576 + 192 = 783` bytes total: 11-byte datum prefix, 772-byte quantized payload, and 768-byte `code_bytes` section.

### FR-001-AC-3: Binary layout correctness
Pack/unpack of `(dim, bits, seed, gamma, code_bytes)` SHALL round-trip losslessly for all valid parameter combinations.
