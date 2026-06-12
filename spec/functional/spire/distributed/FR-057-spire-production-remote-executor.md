---
id: FR-057
title: SPIRE Production Remote Executor
type: functional-requirement
artifact_type: FR
status: APPROVED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-056"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-057: SPIRE Production Remote Executor

## Description

Distributed SPIRE SHALL use a production remote executor that resolves sanitized
libpq/TLS connection state, enforces fanout and governance budgets, validates
endpoint identity, propagates timeout/cancel behavior, and returns validated
candidate/tuple batches to the coordinator merge path.

## Executor State Model

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Blocked: budget or descriptor failure
    Pending --> ReadyTransport
    ReadyTransport --> Sent
    Sent --> CandidateReceive
    CandidateReceive --> HeapReceive
    HeapReceive --> ReadyMerge
    Sent --> Failed
    CandidateReceive --> Failed
    HeapReceive --> Failed
    ReadyMerge --> [*]
    Blocked --> [*]
    Failed --> [*]
```

## Behavior

1. Raw conninfo SHALL be resolved inside executor code from
   `conninfo_secret_name` and SHALL NOT be returned through SQL-visible rows,
   logs, or unsanitized errors.
2. TLS/libpq connection policy SHALL preserve libpq security parameters from
   resolved conninfo.
3. Static fanout limits SHALL bound selected remote nodes, total remote PIDs,
   and PIDs per node before socket open.
4. Advisory governance SHALL bound concurrent dispatches globally and per
   remote node using the reserved SPIRE advisory-lock namespace.
5. Endpoint identity validation SHALL bind coordinator descriptor state to
   remote index identity, extension version, served epoch, tuple transport
   capability, and schema fingerprint state.
6. Strict mode SHALL fail the query when required remote work is stale,
   unavailable, overloaded, or identity-incompatible.
7. Degraded mode MAY skip failed remote work only when the selected query path
   permits degraded execution and SHALL report one row per skipped node.
8. PostgreSQL interrupt/cancel and configured connect/statement timeouts SHALL
   move affected remote work to explicit failure states.
9. Diagnostic functions SHALL distinguish dry planning surfaces from live
   libpq/TLS surfaces that open sockets.

## Workflow

```mermaid
sequenceDiagram
    participant Coord as Coordinator merge path
    participant Exec as Production remote executor
    participant Gov as Fanout budgets + advisory governance
    participant Sec as Secret resolution
    participant Remote as Remote node (libpq/TLS)

    Coord->>Exec: selected remote nodes and PIDs
    Exec->>Gov: check static fanout limits (nodes, total PIDs, PIDs per node)
    Exec->>Gov: acquire advisory dispatch slots (global and per node)
    alt budget or descriptor failure
        Gov-->>Exec: refuse before socket open
        Exec-->>Coord: Blocked (no conninfo exposure)
    else transport ready
        Exec->>Sec: resolve raw conninfo from conninfo_secret_name (executor-internal)
        Exec->>Remote: connect with preserved libpq/TLS security parameters
        Exec->>Remote: validate endpoint identity (index identity, extension version, served epoch, tuple transport capability, schema fingerprint)
        Exec->>Remote: dispatch typed remote_scan_v1 request (Sent)
        Remote-->>Exec: candidate batch (CandidateReceive)
        Remote-->>Exec: heap tuple payloads (HeapReceive)
        Exec->>Exec: validate payload arity, types, and capacity limits
        Exec-->>Coord: validated candidate/tuple batches (ReadyMerge)
    end
    note over Exec,Remote: timeout, interrupt/cancel, transport failure, stale epoch, or identity mismatch moves the affected work to Failed
    alt strict mode
        Exec-->>Coord: query fails when required remote work is stale, unavailable, overloaded, or identity-incompatible
    else degraded mode permitted by the query path
        Exec-->>Coord: skip failed remote work, one reported row per skipped node
    end
```

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-057-AC-1 | Remote execution exposes pending, transport-ready, sent, receive, ready, failed, blocked, strict, and degraded statuses with stable labels | Test |
| FR-057-AC-2 | Budget and governance overload fail before raw conninfo exposure and before candidate batches enter merge state | Test |
| FR-057-AC-3 | Endpoint identity mismatch, stale epoch, timeout, cancellation, transport failure, and degraded skip are observable through operator diagnostics | Demonstration |

### FR-057-AC-1: Executor status labels

Remote execution exposes pending, transport-ready, sent, receive, ready, failed,
blocked, strict, and degraded statuses with stable labels.

### FR-057-AC-2: Pre-socket budget failure

Budget and governance overload fail before raw conninfo exposure and before
candidate batches enter merge state.

### FR-057-AC-3: Fault observability

Endpoint identity mismatch, stale epoch, timeout, cancellation, transport
failure, and degraded skip are observable through operator diagnostics.

## Dependencies

- **Upstream**: FR-055 (topology, remote descriptors, and
  `conninfo_secret_name` placement state), FR-056 (typed remote endpoint and
  tuple transport the executor dispatches).
- **Downstream**: FR-058 (distributed CustomScan read), FR-059
  (coordinator-routed DML/2PC), and FR-060 (diagnostics and operator
  surface), per their declared dependencies on this FR.
