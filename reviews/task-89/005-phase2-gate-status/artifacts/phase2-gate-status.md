# Task 89 Phase 2 Gate Status

Date: 2026-06-08

Head inspected: `ab4e601c9`

## Current State

Task 89 is prepared for implementation, but Phase 2 code work is still gated.

The task file requires:

> ADR must be reviewer-approved before Phase 2 starts.

Current packet state:

| Packet | Purpose | Status |
| --- | --- | --- |
| `001-format-design-adr` | ADR-076 format-design gate | Review requested; no feedback present in checkout |
| `002-cross-am-inventory` | all-AM code inventory | Published |
| `003-task86-extraction-map` | preserved Task 86 TQ+ code extraction map | Published |
| `004-validation-matrix` | all-AM validation matrix and real10k suite template | Published |

Feedback scan:

```text
find reviews/task-89 -path '*/feedback/*' -type f
```

Result: no feedback files.

## Ready Inputs

The following are ready for the first post-approval implementation slice:

- ADR-076 production surface:
  - `storage_format=turboquant`
  - `turboquant_profile=standard|tqplus`
- Shared math extraction map:
  - reintroduce `TqPlusCalibration`;
  - reintroduce calibrated no-QJL 4-bit encode/query/score helpers;
  - preserve non-finite/zero-scale and no-QJL 4-bit validation.
- All-AM inventory:
  - IVF/SPIRE/HNSW need profile and calibration persistence;
  - DiskANN first needs baseline TurboQuant support before TQ+.
- Validation matrix:
  - DBPedia 10k/50k/100k across IVF/SPIRE/HNSW/DiskANN;
  - one non-DBPedia all-AM corpus;
  - streaming-insert drift at 10%/25%/50%.

## Next Implementation Commit After Approval

The first code commit after reviewer approval should be intentionally narrow:

1. Add shared TQ+ math to `src/quant/prod.rs`.
2. Add unit tests for calibration validation, no-QJL 4-bit gating, and score
   equivalence from encoded struct versus raw parts.
3. Avoid AM reloptions, page layouts, and storage metadata in that first code
   commit.

That slice advances all AMs because every AM port depends on the same
calibration/encode/query/score implementation.

## Gate Condition

Do not begin Phase 2 code porting until packet `001-format-design-adr` has an
outside reviewer feedback file approving ADR-076 or explicitly authorizing a
different Phase 2 starting shape.

If reviewer feedback requests ADR changes, process that before code.
