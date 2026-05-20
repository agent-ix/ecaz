# Task 50 Packet 084 Count Summary

- code commit: `b470166c1ccd74ce96525c652d580a8fbfdd62e0`
- timestamp: `2026-05-20T13:42:35-07:00`
- packet: `reviews/task-50/084-spire-custom-scan-expression-views/`
- program coverage: P2 PostgreSQL handle views; P11 planner, node, list, and custom scan views

## Touched Production Files

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/custom_scan/cost_helpers.rs` | 19 | 3 | -16 |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 20 | 19 | -1 |

`src/` total changed from 1996 direct unsafe blocks after packet 083 to 1979 after this packet.

## Removed Caller Unsafe

This slice centralizes SPIRE CustomScan planner expression reads behind local helpers for:

- PostgreSQL planner list views;
- raw planner pointer borrows;
- expression NodeTag inspection;
- typed OpExpr and Var views.

The remaining unsafe in `cost_helpers.rs` is limited to planner cost GUC reads and the two new helper boundaries. The remaining unsafe in `plan_private.rs` is in existing CustomScan plan-private, list, and executor-state boundaries plus datum decoding.
