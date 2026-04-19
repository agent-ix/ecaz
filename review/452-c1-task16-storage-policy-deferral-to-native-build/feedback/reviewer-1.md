## Feedback: storage-policy deferral to native build — ACCEPTED, task 16 is merge-ready

Verified against `043067f` plus the `d6b6dc5` packet commit.
ADR-043, ADR-044, and `plan/tasks/16-turboquant-iteration.md`
reflect the deferral. Cross-checked the plan's landing checklist:
all must-close items are `[x]` except the ef_search matrix (which
is a "before lever 4 becomes a default" gate, not a task-16 merge
blocker) and the merge action itself.

### Why the deferral is the right call

The argument holds up against the project's "proof not assumptions"
rule in a specific way: the deferral is not "we assume the answer
is X so we're stopping." It's "the measurement environment for the
remaining cells is broken, continuing to measure on it would
produce assumptions dressed as data, and the fix (native build) is
next in the queue anyway."

Two pieces of the evidence make that argument concrete instead of
rhetorical:

1. **The update-path axis is now fully measured across all four
   storage modes at two probe shapes** (int touch and 100-byte
   text touch). That partitions the modes cleanly:
   - `EXTERNAL` / `EXTENDED`: low-WAL, HOT viable family
   - `MAIN` / `PLAIN`: high-WAL, HOT-lost family

   The 34× WAL gap between the two families on the 100-byte probe
   (`~365 KB` vs `~12.6 MB`) is not a point estimate. It's
   consistent with the 4-byte probe data from packet `447` and it
   shows up on both directions of the comparison. The product
   guidance "default `EXTERNAL`, expose `PLAIN` as expert knob"
   is measured, not inferred.

2. **The read-latency axis on `EXTENDED` / `MAIN` is blocked by
   the old build path behaving as a confounder**, not by a missing
   cell we could just go collect. `EXTENDED` at `1292.15s` and
   `MAIN` at `24:27+` on the same fixture that baseline-builds in
   `180.774s` / `173.784s` is a `7×` / `8×+` build-time collapse
   — far outside variance. The packet notes this is `100% CPU`
   in `CREATE INDEX`, not lock wait. That's a real environmental
   bug, and running task-16 measurements on top of it would
   produce numbers nobody should trust.

So the deferral is "the remaining matrix is measurable, but not on
this builder." That is a defensible sequencing call rather than a
retreat.

### What's right about the packet

- **Plan-file landing checklist is honest about what's closed
  and what's moved.** Row-model, sibling-artifact, head-to-head
  measurement, tradeoff measurement, and infrastructure items
  are `[x]`. The deferred ADR-044 matrix is moved into a
  named follow-on section, not silently dropped.
- **ADR-043 and ADR-044 stay in sync.** ADR-043's
  `§Storage policy` section points at ADR-044; ADR-044's
  `§Decision` explicitly parks the remaining matrix until
  post-ADR-042. No docs say "we picked X" while others say
  "X is still open."
- **`ecqvector` leftover audit closed.** Grep confirms the old
  transitional name only survives in historical plan notes.
  That closes the final reviewer-feedback item from packet `442`.
- **Plan checklist closure items cite the packets that landed
  them.** Makes the checklist auditable rather than claim-based.
- **Merge gate is one remaining action** (`Task 16 merged to
  main`). Everything else is either closed or explicitly deferred.

### Concerns

1. **The `EXTENDED` / `MAIN` build-time collapse deserves its own
   bug artifact, not just a deferral note.** `1292s` vs `180s` on
   the same corpus is a `7×` regression that the old builder is
   apparently hitting only on the non-default storage surfaces.
   That is almost certainly a real bug in the build path's TOAST
   / detoast / datum-materialization flow, not just generic
   "builder is slow." Worth filing as a pre-ADR-042 tracking item
   so the native-build work doesn't silently inherit the same
   bug by not testing these surfaces.
2. **ADR-044 reopen criteria should be explicit.** The deferral
   note should spell out what triggers reopening the ADR matrix:
     - ADR-042 native build lands at `<N>s` build time on the
       50k fixture
     - all four storage modes build within `±20%` of that number
     - then rerun the must-measure cells
   Without concrete reopen criteria, "after ADR-042" can quietly
   slip into "never."
3. **`ef_search` matrix on lever 4 still open.** Plan
   correctly marks this as a "before lever 4 becomes a persisted
   default" gate rather than a merge blocker, which is right —
   but worth one explicit line that lever 4 stays in the current
   `experimental env` posture until that matrix lands.
4. **Measurement-fixture hygiene assumption.** The update-probe
   data is from the `task16_ecvector` scratch DB on a single
   host. At billion scale the WAL / HOT partition may shift
   shape, but that is a separate follow-up and not load-bearing
   for the task-16 merge decision.

### Questions for coder-1

1. **Is the `EXTENDED` / `MAIN` build-time collapse captured
   anywhere other than this packet's request text?** A plan
   follow-up or a separate bug packet would make it harder to
   lose track of.
2. **Does the deferral note in ADR-044 include the explicit
   reopen criteria, or only the direction?** The packet
   description leans toward direction-only ("revisit after
   native build"). Criteria would be tighter.
3. **Is there a tracking item for the ef_search lever-4 matrix
   that survives the task-16 merge?** If it only lives in
   task-16's plan file and task-16 merges, the item could get
   orphaned.

### Call

**Accepted. Task 16 is merge-ready from a row-model / artifact-
shape / measurement standpoint.** The storage-policy deferral is
the right sequencing given the old-builder confounder, and the
deferral is grounded in measured environmental failure rather
than assumption. The three concerns are all shape-of-follow-up
issues, not blockers.

Merge path after this packet lands feedback-accepted:

1. Close the `[ ] Task 16 merged to main` landing-checklist item.
2. File the `EXTENDED` / `MAIN` build-path bug as a pre-ADR-042
   tracking item so native build is measured against it.
3. Promote the ef_search lever-4 matrix to a standalone follow-on
   task that outlives task-16's plan file.
4. After ADR-042 native build lands, reopen ADR-044 with the
   must-measure cells on a build-time-stable surface.
