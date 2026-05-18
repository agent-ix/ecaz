## Feedback: ecvector surface head-to-head measurement — ACCEPTED

Verified against `b704d7a`. Fresh scratch DB (`task16_ecvector`),
50k real corpus, m=16 ef_search=128 q200, `warm-after-prime3`
cache, `cached-plan` timing, confirming reruns on both inline cells.

### What's right

- **Fresh scratch DB eliminates the stale-extension class of bug.**
  The old `postgres` scratch DB still had pre-`442` catalog. Running
  on a stale DB would have measured a product surface nobody ships.
  The `task16_ecvector` DB + `CREATE EXTENSION tqvector` from
  current head is the authoritative fixture for the canonical-row
  measurement arc.
- **Three-question structure is the right shape.** Default-storage,
  inline-storage, and head-to-head against PqFastScan on the same
  inline surface are the three questions task-16 had open. Answering
  them in one packet on one DB isolates the variable cleanly.
- **Planner-isolation discipline.** The note that the planner may
  pick the TurboQuant index for a PqFastScan cell when both coexist,
  and the discipline of dropping the competing index before the
  comparison, is correct. The verified wrapper caught a wrong
  measurement on the first default-surface PqFastScan attempt — that
  is the wrapper doing its job.
- **Confirming reruns on both inline cells.** Inline TurboQuant
  `3.427ms` → `3.195ms` and inline PqFastScan `2.987ms` → `2.954ms`.
  The second run is the quoted number in both cases, which is the
  right discipline — first-run warm cache effects can flatter the
  measurement.
- **Recall stayed pinned across surfaces.** `graph_recall_at_10`
  and `ndcg_at_10` are bit-identical between default and inline on
  both formats. That locks in "inline storage is a layout lever,
  not a scoring lever." If recall had moved, the measurement would
  be invalidated.

### Key findings (worth pulling out)

1. **Default-storage `ecvector` does not inherit the packet-`441`
   win.** TurboQuant `5.248ms`, PqFastScan `5.904ms`. The naive
   product story ("just use `ecvector`") gets you the correct row
   model but not the serious-lane latency yet. This is a real
   product-surface finding the task should not gloss.
2. **Inline-storage `ecvector` recovers `-39.12%` on the confirming
   TurboQuant rerun** (`5.248ms → 3.195ms`). Lever lives in
   `attstorage`, not in the type. Productizing the lever means
   picking a default for the `ecvector` `SET STORAGE` choice, or
   exposing it as a documented column-level knob.
3. **On the same inline surface, PqFastScan still leads TurboQuant
   by 7.54%** (`2.954ms` vs `3.195ms`) with slightly higher recall
   (`0.9635` vs `0.9629`). Task-16's stated goal is "TurboQuant
   remains credible" — this reading supports that: inline-`ecvector`
   TurboQuant is in the same latency class as PqFastScan at this
   operating point. It is not faster, but it is no longer a bad
   second choice.

### Concerns

1. **`m=16 ef_search=128` is a single operating point.** ADR-043 and
   the task-16 plan both ask for an ef-search matrix. This packet
   closes the canonical-surface head-to-head at the default
   operating point, but the plan's ef-search-matrix checklist item
   is still open and needs its own packet. Worth calling out that
   446 ≠ full matrix.
2. **No build-time comparison in this packet.** Default vs inline
   build is measured in packet `447`, but a TurboQuant-vs-PqFastScan
   build-time cell on the inline surface would also be useful.
   Current packet compares query-side only.
3. **Planner-isolation technique is documented but not automated.**
   The "drop competing index before measuring" pattern is manual
   choreography. The packet flags this: a bench helper with an
   index-disable knob would make future same-table multi-format
   comparisons less awkward. Reasonable follow-on, not a blocker.
4. **Scratch DB is single-host and single-run.** At billion-scale
   the storage tradeoff inline enforces may change character. The
   50k-warm-real seam answers the serious-lane question honestly at
   that scale; it does not generalize to billion without separate
   measurement.

### Questions for coder-1

1. **Does the `cached-plan` timing mode strip out plan-compilation
   variance enough that the `2.954ms`/`3.195ms` reruns are
   effectively noise-free?** A one-line confirmation that the
   timing-mode contract is what I think it is would close the loop
   on "is this 7.54% delta real or planner noise."
2. **Does the fresh `task16_ecvector` DB ship with the same
   `shared_buffers` sizing as the production-representative
   default?** Buffer-cache sizing affects both surfaces, and the
   inline-storage heap working set is `305%` of `shared_buffers`
   (packet `447`). If the bench DB is under-buffered relative to
   deployment, the inline win may be understated.
3. **Any reason to measure the default-storage surface with
   `SET STORAGE EXTENDED` vs `EXTERNAL`?** `e` / `x` storage modes
   compress differently; the measurement used `e`. Not necessarily a
   problem, just worth confirming that is the intended product
   default for `ecvector`.

### Call

Accepted. This is the canonical task-16 head-to-head measurement
on the corrected row type. Three product-shaping findings land
cleanly:

- `ecvector` alone does not buy the serious-lane win
- `ecvector` + inline storage does
- TurboQuant on that surface is credible-but-second vs PqFastScan

Remaining task-16 measurement work is the ef-search matrix and the
final lever-policy decision — both tracked in the plan, neither
blocked by this packet.
