---
id: FR-011
title: WAL Safety — GenericXLog Usage
type: functional-requirement
status: APPROVED
object_type: process
traces:
  - US-003
  - US-005
---
# FR-011: WAL Safety — GenericXLog Usage

## Requirement

All page modifications within the `tqhnsw` index access method SHALL use PostgreSQL's GenericXLog facility for crash-safe durability.

### Pattern (from pgvector)

```rust
// Before modifying any page:
let state = GenericXLogStart(index);
let page = GenericXLogRegisterBuffer(state, buffer, flags);

// ... modify page contents ...

GenericXLogFinish(state);  // atomically writes WAL record
```

### Rules

1. No page SHALL be modified outside a GenericXLog transaction
2. If an error occurs between `GenericXLogStart` and `GenericXLogFinish`, the changes SHALL be rolled back automatically (standard GenericXLog guarantee)
3. After a crash and WAL replay, the index SHALL be in a consistent state
4. pgrx wraps these C functions — use the pgrx wrappers

## Acceptance Criteria

### FR-011-AC-1: Crash recovery
After building an index, simulating a crash (kill -9), and restarting PostgreSQL, the index SHALL pass `REINDEX` without errors.

### FR-011-AC-2: No direct page writes
A code audit SHALL confirm that no index page is modified without GenericXLog wrapping.
