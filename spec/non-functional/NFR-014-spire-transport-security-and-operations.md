---
id: NFR-014
title: Distributed Transport Security and Operations
type: NFR
quality_attribute: security
status: PROPOSED
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
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-083"
    type: "constrains"
    cardinality: "1:N"
---
# NFR-014: Distributed Transport Security and Operations

## Statement

SPIRE and ec_distann distributed transport SHALL preserve libpq security
semantics, keep secrets out of data/control payloads, validate endpoint and
schema identity before mutation, bound received payloads before allocation, and
provide explicit operator-visible recovery.

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
5. EC_DISTANN participant-identity configuration, node registration, control identity
   (`ec_distann_control_identity`), unpublished-generation listing
   (`ec_distann_list_unpublished_generations`), build, handoff, topology
   inspection, expansion, materialization, publication, recovery, retirement,
   and abort endpoints SHALL be executable only by the extension owner and an
   explicitly granted internal cluster role; `PUBLIC` SHALL have no EXECUTE privilege.
   In particular, an ordinary reader SHALL NOT acquire publish/reclaim side
   effects merely by observing pending recovery during a scan.
6. EC_DISTANN write-side endpoints SHALL validate participant identity, build
   identity, epoch/manifest digest, row-schema fingerprint, and placement owner
   before mutating storage.
7. EC_DISTANN handoff and row-materialization endpoints SHALL resolve binary
   send/receive functions from validated local catalogs; requests SHALL NOT
   select a function name or OID.
8. EC_DISTANN endpoints SHALL reject declared lengths, array cardinalities, or
   decoded sizes outside the governing FR constraints before allocating the
   corresponding payload.
9. EC_DISTANN errors SHALL expose stable sanitized `EC_*` categories without raw
   conninfo, secret names, row payload bytes, source identities, or unsanitized
   remote errors.
10. EC_DISTANN node registration SHALL persist only a conninfo secret reference;
    raw conninfo SHALL remain in the secret resolver and in-memory connection
    setup path.
11. EC_DISTANN generation catalogs, graph stores, row-tier heaps, TOAST
    relations, and local directories SHALL be extension-owned internal
    relations with no direct `PUBLIC` access. User-visible row access SHALL pass
    through the validated coordinator/materialization path.
12. EC_DISTANN endpoint identities and conninfo secret references SHALL use the
    exact distinct FR-078 grammars that exclude `=`, whitespace, quoting, URI
    schemes, and provider-key aliases. Registration SHALL persist only the
    endpoint identity and canonical index locator returned by the authenticated
    participant endpoint, never caller-only spellings.
13. Desired-roster catalogs SHALL NOT be used to route a Published or retained
    epoch. Every such operation SHALL resolve the immutable private
    build-participant binding selected by the epoch's build id; that private
    catalog remains non-public and never enters manifest or diagnostic bytes.
14. Every EC_DISTANN `SECURITY DEFINER` endpoint SHALL pin `search_path` to
    `pg_catalog`, the extension schema, and explicit `pg_temp` last. Omitting
    `pg_temp` is not equivalent because PostgreSQL otherwise searches the
    session temporary schema first for relation and type names.

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
6. EC_DISTANN build and publish recovery SHALL expose build id, epoch, state,
   participant receipt status, and sanitized node identity through an
   operator-readable status surface.
7. EC_DISTANN abort and force-retire operations SHALL emit auditable records with
   caller, target identity, prior state, and outcome.
8. EC_DISTANN recovery SHALL be operator-driven on an unpublished generation
   unless FR-082's durable commit-only publish decision requires automatic
   completion.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Raw conninfo / raw secret exposure through SQL diagnostics, logs, result rows, or unsanitized errors | zero exposures | no exceptions | inspection and SQL diagnostics over PG18 fixtures |
| Schema drift and endpoint identity mismatch handling | 100% fail closed before mutating remote state | no exceptions | PG18 fixture verification |
| JSON tuple transport on the production distributed read path | not selected once typed transport is required | no exceptions | typed-transport readiness inspection |
| Remote prepared-transaction recovery readiness | `max_prepared_transactions` > 0 with reserved slots; operator-driven reaper documented | no exceptions | SQL diagnostics and readiness hints |
| Unauthorized EC_DISTANN internal endpoint execution | zero successful calls by `PUBLIC` or an ungranted role | no exceptions | PG18 privilege test |
| Raw secret / row-payload / source-identity exposure from EC_DISTANN errors and diagnostics | zero exposures | no exceptions | fault matrix plus log/result inspection |
| Oversize EC_DISTANN handoff/materialization allocation | rejected before declared-size allocation | no exceptions | boundary and malformed-length tests |
| EC_DISTANN schema, endpoint, build, epoch, and owner mismatch handling | 100% fail closed before mutation | no exceptions | PG18 multinode fault drills |

## Verification

Verification SHALL use inspection, SQL diagnostics, and PG18 fixtures for:

- secret non-exposure and sanitized error categories;
- typed transport readiness and JSON production-path retirement;
- `max_prepared_transactions` readiness hints;
- orphaned prepared transaction reaper behavior;
- schema drift fail-closed behavior before remote mutation.
- EC_DISTANN endpoint privilege revocation from `PUBLIC`;
- EC_DISTANN identity/schema/owner validation before mutation;
- EC_DISTANN malformed and oversize payload rejection before allocation;
- sanitized EC_DISTANN status, abort, recovery, and force-retire audit records.

## Acceptance Criteria


| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-014-AC-1 | No SQL-visible remote transport surface exposes raw conninfo or raw remote error text. | Demonstration |
| NFR-014-AC-2 | Remote write readiness and prepared transaction recovery are documented with explicit operator action and failure modes. | Inspection |
| NFR-014-AC-3 | Schema drift and endpoint identity mismatches fail before mutating remote state. | Inspection |
| NFR-014-AC-4 | An unprivileged session cannot execute any EC_DISTANN internal distributed endpoint. | Demonstration |
| NFR-014-AC-5 | Malformed or oversize EC_DISTANN payloads are rejected before storage mutation or allocation beyond the documented cap. | Inspection |
| NFR-014-AC-6 | EC_DISTANN recovery and destructive lifecycle actions are attributable to a caller and target without exposing secrets or row payloads. | Demonstration |

