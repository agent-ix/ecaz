---
id: FR-033
title: IVF Insert, Vacuum, and Admin Snapshots
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: process
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
---
# FR-033: IVF Insert, Vacuum, and Admin Snapshots

## Requirement

`ec_ivf` SHALL support live insert, vacuum cleanup, and read-only admin/debug snapshots for drift and page ownership.

## Behavior

1. Live insert SHALL assign new tuples to a valid posting list without duplicating heap TIDs.
2. Insert SHALL reject dimensional or storage-format mismatches.
3. Vacuum SHALL remove dead heap TIDs from posting lists and update vacuum statistics.
4. Admin snapshots SHALL expose metadata, drift, cost, and page-ownership state for review and tuning.

## Acceptance Criteria

### FR-033-AC-1

Rows inserted after index creation are reachable through the IVF index.

### FR-033-AC-2

DELETE plus VACUUM removes dead heap TIDs from IVF posting lists.

### FR-033-AC-3

IVF admin snapshots reject non-IVF indexes with a clear error.
