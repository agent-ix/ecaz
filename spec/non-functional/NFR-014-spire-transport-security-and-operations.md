---
id: NFR-014
title: SPIRE Transport Security and Operations
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/FR-056"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-059"
    type: "constrains"
    cardinality: "1:N"
---
# NFR-014: SPIRE Transport Security and Operations

## Requirement

SPIRE remote transport and coordinator-routed writes SHALL preserve libpq
security semantics, avoid exposing raw secrets, fail closed on schema drift and
endpoint identity mismatches, and provide operator-owned recovery for remote
prepared transactions.

## Security Constraints

1. Raw conninfo SHALL be resolved from `conninfo_secret_name` inside executor
   code and SHALL NOT be returned through SQL diagnostics, logs, result rows, or
   unsanitized errors.
2. libpq security parameters such as TLS mode, root certificate, client
   certificate, and private-key options SHALL be preserved from the resolved
   conninfo.
3. Authentication and certificate failures SHALL be reported with sanitized
   categories and operator hints, not raw remote error payloads.
4. JSON tuple transport SHALL NOT be selected by the production distributed read
   path once typed transport is required.

## Operational Constraints

1. Remote nodes used for coordinator-routed writes SHALL set
   `max_prepared_transactions` above zero and reserve slots for SPIRE plus
   other prepared transactions.
2. Prepared transaction GIDs SHALL follow the SPIRE GID format in `FR-059`.
3. The reaper SHALL be operator-driven in v1 and SHALL NOT run as an implicit
   background worker.
4. Distributed DDL SHALL follow pause/apply/refresh/resume ordering across
   coordinator and remotes.
5. Coordinator-routed INSERT, UPDATE, and DELETE SHALL compare coordinator and
   remote schema fingerprints before mutating remote SQL.

## Verification

Verification SHALL use inspection, SQL diagnostics, and PG18 fixtures for:

- secret non-exposure and sanitized error categories;
- typed transport readiness and JSON production-path retirement;
- `max_prepared_transactions` readiness hints;
- orphaned prepared transaction reaper behavior;
- schema drift fail-closed behavior before remote mutation.

## Acceptance Criteria

### NFR-014-AC-1

No SQL-visible remote transport surface exposes raw conninfo or raw remote error
text.

### NFR-014-AC-2

Remote write readiness and prepared transaction recovery are documented with
explicit operator action and failure modes.

### NFR-014-AC-3

Schema drift and endpoint identity mismatches fail before mutating remote state.
