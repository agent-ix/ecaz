---
id: FR-003
title: tqvector Binary Protocol (Send/Receive)
type: FR
status: APPROVED
object: api_endpoint
traces:
  - US-001
  - FR-001
---
# FR-003: tqvector Binary Protocol (Send/Receive)

## Description

The extension SHALL register binary send/receive functions (`tqvector_send`, `tqvector_recv`) for efficient client-server transfer (e.g., COPY BINARY, libpq binary format).

### Send Function: `tqvector_send`

- SHALL emit the internal binary representation unchanged (the packed format IS the wire format)

### Receive Function: `tqvector_recv`

- SHALL validate the received bytes (minimum structural payload size, code length matches dim/bits)
- SHALL reject malformed input with ERROR

## Endpoint

The binary protocol surface is registered as two `LANGUAGE c` SQL functions in `sql/bootstrap.sql`. Both are `IMMUTABLE STRICT PARALLEL SAFE`.

| Function | Signature | Direction | Description |
|---|---|---|---|
| `tqvector_send` | `tqvector_send(tqvector) → bytea` | tqvector → bytea | Emits the internal packed representation unchanged — the on-disk packed format IS the wire format. |
| `tqvector_recv` | `tqvector_recv(internal) → tqvector` | internal (wire bytes) → tqvector | Reads the wire bytes via `unpack`, validating the minimum structural size (`MIN_BINARY_BYTES = 6`: 2-byte `dim` + 4-byte `gamma`) and that the trailing code length matches `code_len(dim, bits)`; rejects malformed input with ERROR. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | `tqvector_recv(tqvector_send(val))` produces a value identical to `val` for all valid values | Test |
| FR-003-AC-2 | Binary payloads shorter than 15 bytes raise ERROR | Test |

### FR-003-AC-1: Binary round-trip
`tqvector_recv(tqvector_send(val))` SHALL produce a value identical to `val` for all valid tqvector values.

### FR-003-AC-2: Reject truncated binary
A binary payload shorter than 15 bytes SHALL raise ERROR. This threshold covers the 11-byte datum prefix plus the required 4-byte `gamma` field; `code_bytes` are validated separately from `dim` and `bits`.

## Dependencies

- **Upstream**: US-001, FR-001 (traces)
- **Downstream**: none identified
