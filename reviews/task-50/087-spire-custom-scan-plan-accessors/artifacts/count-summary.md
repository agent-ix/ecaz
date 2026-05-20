# Task 50 Packet 087 Count Summary

- code commit: `cc3d94cc30cc9bd8fad66f364a0a3845b3d66278`
- timestamp: `2026-05-20T13:56:31-07:00`
- packet: `reviews/task-50/087-spire-custom-scan-plan-accessors/`
- program coverage: P10 scan opaque/raw ownership contracts; P11 planner, node, list, and custom scan views

## Touched Production Files

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/custom_scan/explain.rs` | 5 | 3 | -2 |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 10 | 10 | 0 |

`src/` total changed from 1963 direct unsafe blocks after packet 086 to 1961 after this packet.

## Removed Caller Unsafe

This slice makes checked SPIRE CustomScan plan accessors safe to call:

- `custom_scan_plan`
- `custom_scan_mode_from_plan`
- `custom_scan_index_oid_from_plan`

That removes explain-callback caller unsafe around plan extraction and index-OID lookup while keeping plan-private list decoding in the existing named helper boundary.
