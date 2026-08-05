# Task 02: Datum and I/O

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: complete

## Scope

Implement the `tqvector` datum layout, type registration, text I/O, and binary send/receive.

## Owns

- `FR-001`
- `FR-002`
- `FR-003`

## Dependencies

- Quantizer payload conventions from Task 01

## Unblocks

- SQL function bindings
- end-to-end type storage tests

## Deliverables

- Datum pack/unpack
- Type registration
- `tqvector_in`
- `tqvector_out`
- `tqvector_send`
- `tqvector_recv`

## Primary Tests

- `TC-001` to `TC-007`
- `TC-101` to `TC-104`

## Notes

- Binary layout is a shared interface. Coordinate carefully before downstream code starts depending on it.
