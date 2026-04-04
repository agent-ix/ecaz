# Task 02: Datum and I/O

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
