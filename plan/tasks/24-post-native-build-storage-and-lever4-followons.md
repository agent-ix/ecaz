# Task 24: Post-Native-Build Storage Policy and Lever-4 Follow-Ons

Status: blocked on ADR-042 native HNSW build

Follow-on to ADR-042 and ADR-044.

## Why this task exists

Task 16 intentionally stopped short of closing two performance-policy questions:

1. the remaining `ecvector` storage-policy matrix (`EXTENDED`, `MAIN`,
   `PLAIN + fillfactor`, C1 index-owned raw-f32 payload)
2. the lever-4 (`full_lut`) `ef_search` matrix that must land before lever 4
   can become a persisted default

Those items are not missing because the project forgot them. They were deferred
because the outgoing pre-ADR-042 build path became the confounder.

Quiet-window evidence from task 16:

- known old-builder baselines:
  - `EXTERNAL` default build: `180.774s`
  - `PLAIN` build: `173.784s`
- non-default storage surfaces on the same old builder:
  - `EXTENDED`: `1292.15s` before termination
  - `MAIN`: exceeded `24:27` before termination

That is a real tracking item, not normal variance. Native build must be
measured against it rather than letting it disappear into "the old builder was
slow."

## Scope

After ADR-042 native HNSW build lands and the 50k fixture has a stable build
surface again:

- rerun the deferred ADR-044 storage-policy cells
- verify whether the old-builder `EXTENDED` / `MAIN` build collapse disappears
  or persists
- run the lever-4 `ef_search` matrix and make the persisted-default decision

## Entry criteria

Do not start this task until all of the following are true:

1. ADR-042 native HNSW build is landed on the branch under test.
2. The 50k real-corpus fixture can build the current default `EXTERNAL`
   TurboQuant index reliably enough to establish a stable baseline.
3. Non-default storage surfaces (`EXTENDED`, `MAIN`) build within the same
   rough order of magnitude as that baseline on the native builder, or the
   remaining gap is explained by profiling rather than mystery.

Practical reopen criteria for ADR-044:

- establish a fresh native-build baseline on the 50k fixture for
  `EXTERNAL`
- build `EXTENDED`, `MAIN`, and `PLAIN` on the same native builder
- if the non-default surfaces are within about `±20%` of the new builder's
  stable baseline, rerun the deferred q200/storage-policy matrix
- if they are still far outside that band, file the build collapse as an
  active builder bug and resolve that before trusting any storage-policy
  conclusions

## Work items

### A. Builder sanity / bug carry-forward

- [ ] Reproduce the task-16 `EXTENDED` / `MAIN` build collapse on the native
  builder.
- [ ] If the collapse is gone, record that the old-builder pathology was
  builder-specific and close the bug note.
- [ ] If the collapse persists, profile and root-cause it before using
  `EXTENDED` / `MAIN` q200 results to drive product policy.

### B. ADR-044 storage-policy matrix

- [ ] `EXTENDED` q200 serious-lane cell on native build
- [ ] `MAIN` q200 sanity cell on native build
- [ ] `PLAIN + fillfactor` sweep (`70 / 80 / 90`)
- [ ] detoast-vs-decompress decomposition if still informative
- [ ] C1 index-side cold-page rerank-payload sketch, updated against the
  native-build implementation seams

### C. Lever-4 follow-on

- [ ] Run the lever-4 `full_lut` quantized-lane `ef_search` matrix at
  `64 / 128 / 256`
- [ ] Decide whether lever 4 remains:
  - experimental env-only
  - a persisted reloption
  - the default exact-score mode on supported hardware
- [ ] Keep lever 5 (`int8_approx`) in the "not justified on current x86"
  posture unless a different hardware lane proves otherwise

## Deliverables

- measurement packet for the reopened ADR-044 matrix
- explicit bug disposition for the old task-16 `EXTENDED` / `MAIN` build
  collapse
- packet recording the lever-4 `ef_search` matrix and the persisted-default
  decision

## Out of scope

- changing task 16's merge readiness
- changing ADR-043's accepted row-model / type taxonomy
- retrofitting the old builder just to make these cells easier to measure
