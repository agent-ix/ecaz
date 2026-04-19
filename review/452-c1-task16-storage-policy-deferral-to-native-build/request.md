# Review Request: C1 Task16 Storage-Policy Deferral to Native Build

Current head at execution: `043067f`

## Context

Task 16's row-model / artifact-shape work is landed:

- `ecvector(dim)` is the canonical row type
- `tqvector` is the narrow TurboQuant-family sibling artifact
- the real `ecvector` head-to-head is measured on the canonical surface
- the inline-storage tradeoff is measured and documented

What remained open on paper was ADR-044:

- `EXTENDED` q200 cell
- `MAIN` sanity cell
- `PLAIN + fillfactor` sweep
- larger touched-column update probe
- build-path / index-side follow-up reasoning

The branch now has enough evidence to decide that this work should **not**
block task 16 merge.

## Additional evidence gathered in this slice

### 1. Storage-code correction was real, not cosmetic

The live pg17 scratch probe confirmed:

- `EXTERNAL` => `attstorage = 'e'`
- `EXTENDED` => `attstorage = 'x'`
- `MAIN` => `attstorage = 'm'`
- `PLAIN` => `attstorage = 'p'`

So packet `446` / packet `447` were already measuring the current-head
`EXTERNAL` default, not `EXTENDED`.

### 2. The update path is no longer ambiguous

The larger touched-column probe is now broad enough to preserve the current
guidance without more task-16 work on heap storage policy:

Steady 1k-row batch, 100-byte text touch:

| Surface | WAL bytes | HOT |
|---------|-----------|-----|
| `EXTERNAL` / current default | `369,048` | `27` |
| `EXTENDED` | `364,144` | `40` |
| `MAIN` | `12,661,104` | `0` |
| `PLAIN` | `12,642,488` | `0` |

And the int-touch probe on the new surfaces showed the same shape:

| Surface | WAL bytes | HOT |
|---------|-----------|-----|
| `EXTENDED` | `267,192` | `65` |
| `MAIN` | `12,645,072` | `0` |

So the policy-level conclusion is already stable:

- `EXTERNAL` / `EXTENDED` are the low-WAL, HOT-viable family
- `MAIN` / `PLAIN` are the high-WAL, HOT-lost family

That is enough to keep the current product guidance:

- current default stays `EXTERNAL`
- `PLAIN` remains the expert read-mostly lever

### 3. The old build path is now the confounder

Quiet-window TurboQuant builds on the current pre-ADR-042 builder were far
outside the established task-16 baseline band:

Earlier baselines:

- `EXTERNAL` default: `180.774s`
- `PLAIN`: `173.784s`

Quiet-window follow-up:

- `EXTENDED` build: `1292.15s` before termination
- `MAIN` build: exceeded `24:27` before termination

The backend checks showed these were not lock waits; both were `100%` CPU in
`CREATE INDEX`.

That means the remaining storage-policy work is now entangled with the outgoing
builder itself. Continuing to optimize `ecvector` heap policy on top of that
old path is a poor use of task-16 time when ADR-042 native HNSW build is next
specifically to get build throughput under control.

## What this slice changes

Updated:

- `spec/adr/ADR-044-ecvector-rerank-source-location-and-storage-policy.md`
- `plan/tasks/16-turboquant-iteration.md`

Decision captured in docs:

- task 16 no longer blocks on closing ADR-044
- current-head `EXTERNAL` default remains the product answer for now
- `PLAIN` remains the documented expert lever
- the remaining ADR-044 matrix is explicitly deferred until after
  ADR-042/native build
- task 16's landing checklist now treats the row-model / artifact-shape
  refactor as merge-ready, with the storage-policy work moved into a deferred
  follow-on section

Also cleaned up stale plan state:

- ADR-043 ratification item is now marked closed, since ADR-043 is already
  `ACCEPTED`
- the stale `ecqvector` doc/error-text blocker is closed; grep now shows the
  old name only in historical plan notes

## Why this is the right cut

This is not "giving up on the storage question." It is sequencing.

The branch already has enough to land the structural work:

- canonical exact row type
- sibling artifact type
- real `ecvector` head-to-head numbers
- documented `EXTERNAL` vs `PLAIN` tradeoff
- enough new update-path data to know `MAIN` tracks the bad side of that
  tradeoff on writes

What it does **not** have is a trustworthy old-build-path environment for
further optimization conclusions. Native build is the next dependency precisely
because the build path is not under control yet.

So the correct cut is:

1. land task 16 and the shape refactor now
2. land native build
3. come back to ADR-044 on top of a faster, more controllable builder

## Validation

Docs / plan only.

No Rust or SQL code changed in this slice, so the cargo / pgrx / clippy
checkpoint trio was not rerun.

## Review focus

1. Does the docs change make the deferral explicit enough that task 16 can be
   treated as merge-ready without hand-waving?
2. Is the reasoning sound that the new update-path data is sufficient to keep
   `EXTERNAL` as the default and `PLAIN` as the expert knob for now?
3. Is deferring the remaining ADR-044 matrix until after ADR-042/native build
   the right cut, given the `EXTENDED` / `MAIN` old-builder behavior?
