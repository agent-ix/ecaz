# Task 50 Packet 085 Count Summary

- code commit: `9865b278258228e90ea8f08701443c6e7cc331ae`
- timestamp: `2026-05-20T13:46:48-07:00`
- packet: `reviews/task-50/085-spire-custom-scan-plan-private-helpers/`
- program coverage: P10 scan opaque/raw ownership contracts; P11 planner, node, list, and custom scan views

## Touched Production Files

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 19 | 10 | -9 |

`src/` total changed from 1979 direct unsafe blocks after packet 084 to 1970 after this packet.

## Removed Caller Unsafe

This slice centralizes SPIRE CustomScan plan-private metadata construction and reads behind:

- a checked `custom_scan_custom_private` accessor;
- safe plan-private list builders for copied string metadata;
- safe counted-column and PK-column offset helpers.

The remaining unsafe in `plan_private.rs` is concentrated in PostgreSQL list/node reads, string-node decoding, datum decoding, path-private reads, and the test-only deep-copy roundtrip.
