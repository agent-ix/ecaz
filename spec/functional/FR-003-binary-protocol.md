---
id: FR-003
title: tqvector Binary Protocol (Send/Receive)
type: functional-requirement
status: APPROVED
object_type: api
traces:
  - US-001
  - FR-001
---
# FR-003: tqvector Binary Protocol (Send/Receive)

## Requirement

The extension SHALL provide binary send/receive functions for efficient client-server transfer (e.g., COPY BINARY, libpq binary format).

### Send Function: `tqvector_send`

- SHALL emit the internal binary representation unchanged (the packed format IS the wire format)

### Receive Function: `tqvector_recv`

- SHALL validate the received bytes (minimum structural payload size, code length matches dim/bits)
- SHALL reject malformed input with ERROR

## Acceptance Criteria

### FR-003-AC-1: Binary round-trip
`tqvector_recv(tqvector_send(val))` SHALL produce a value identical to `val` for all valid tqvector values.

### FR-003-AC-2: Reject truncated binary
A binary payload shorter than 15 bytes SHALL raise ERROR. This threshold covers the 11-byte datum prefix plus the required 4-byte `gamma` field; `code_bytes` are validated separately from `dim` and `bits`.
