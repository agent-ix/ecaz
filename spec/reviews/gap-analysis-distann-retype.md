---
id: SR-008
title: "Retroactive gap analysis: merged-unreviewed PRs #73 (marker retirement) and #75 (distann.md TestMatrix -> index retype)"
type: SpecReview
analysis: gap-analysis
scope: "PR #73 (spec/tests.md / TM-001) and PR #75 (spec/matrix/distann.md / TM-002), verified at main c2d943730 with quire 0.23.0 and the installed spec-artifacts-process module contract"
review_set: subset
---
# SR-008: Retroactive Gap Analysis — PRs #73 and #75 (DistANN matrix retype)

## Summary

Retroactive review of two self-merged, unreviewed PRs, verified against the
merged tree at `main` `c2d943730` with `quire 0.23.0` and the installed
`spec-artifacts-process` module manifest. Both PRs do what their bodies claim,
and both bodies are honest about their trades. The verified picture:

- **PR #73** (`spec/tests.md`, TM-001): normalized nine rows — seven `⚠️` and
  two `🟡` — to `🚧`, and dropped the TC-018 row. All three mechanisms the PR
  body cites check out against the module contract and the engine source
  (details below). The normalization is coverage-neutral as claimed. Dropping
  TC-018 instead of repairing it was a judgment call, mitigated because
  **GAP-004 in the same file verifiably tracks the identical work**
  (`spec/tests.md:489`: "HNSW insert throughput decontention — Track as future
  Task 13 work", the same subject and task the dropped row named).
- **PR #75** (`spec/matrix/distann.md`, TM-002): retyped the document from
  `type: TestMatrix` to `type: index`. Verified: before the retype the file was
  the bundle's only failing document (3 archetype findings, reproduced below);
  after it, `quire validate spec/matrix/distann.md` and
  `quire validate --scope . 'spec/**/*.md'` are **completely silent about the
  file** (bundle exit 0, grammar warnings on other files only). The 192 AC/CON
  rows are validated by no archetype. The PR body disclosed exactly this trade,
  and the deferred destination (nested module matrix, the `filament-ide-rs`
  pattern) **is now ticketed as agent-ix/ecaz#76** (open, filed 2026-08-21
  15:43Z — about ten hours after the merge, so the "no ticket" gap this review
  was seeded with existed but is closed).
- New findings beyond the merged PRs' own disclosures are in the Findings
  table: the retired `🟡`/`⚠️` vocabulary lives on inside the now-unvalidated
  TM-002 (6 `🟡` + 47 `⚠️` occurrences), TM-002's Coverage Summary breakdown no
  longer reconciles with its own rows, the in-file banner points at the closed
  symptom ticket #74 rather than the live decision ticket #76, and the `TM-002`
  identity stays claimed by a `type: index` document.

This document validates clean as `SpecReview` under the installed module set;
no vocabulary carve-out was needed for it (ecaz's own trace-vocabulary
divergence, per agent-ix/quire-rs#211, did not surface here).

## PR #73 — what was verified

Merged 2026-08-21 04:30Z as `402150362`, touching only `spec/tests.md`
(+9/-10). Three claims, all verified:

1. **`🟡` was a hard failure, not a style choice.** The TestMatrix contract's
   Status column pattern is `^(✅|⚠️|❌|🚧|⛔)(\s+.*)?$`
   (`spec-artifacts-process` manifest, `test_cases` assert). `🟡` is outside
   the pattern, so TC-043/TC-044 failed structural validation. Rewriting to
   `🚧` with the note preserved is the minimal conforming fix.
2. **`⚠️` -> `🚧` is coverage-neutral.** The module's status classes are
   `complete: ["✅"]`, `pending: ["🚧"]`, `failed: ["❌"]`, `retired: ["⛔"]`
   (manifest lines 825–830) — `⚠️` is admitted by the column pattern but
   belongs to no class, so `StatusVocabulary::class_of` (quire-rs
   `src/traceability.rs:477`) returns `Unknown` for it. The status-lie
   consumer asks `== Complete` (quire-rs `src/coverage.rs:626`), and `Unknown`
   is also what the `undeclared_statuses` backstop reports
   (`src/coverage.rs:576-580`). Neither `Unknown` nor `Pending` classes as
   complete, so the seven `⚠️ Partial` rows (TC-005, TC-006, TC-013, TC-014,
   TC-034, TC-035, TC-036) claim exactly as much after as before. Verified
   residue: zero `⚠️`/`🟡` remain in `spec/tests.md`.
3. **TC-018 could not stay as written.** Its `Traces To` cell was
   `Future Task 13` — spaces and no `<KIND>-<N>` token, refused by the
   contract's Traces To pattern, which deliberately admits no placeholder. No
   HNSW-insert-throughput requirement exists in the spec for it to trace to.
   Deletion (rather than minting the missing requirement and repairing the
   row) is the judgment call; **GAP-004 verifiably tracks the same work**
   (`spec/tests.md:489`), so the work item survives the row. Recorded as
   FND-001, not treated as a defect to reverse.

## PR #75 — what was verified

Merged 2026-08-21 05:27Z as `c2d943730`, touching only `spec/matrix/distann.md`
(+44/-3): frontmatter `type: TestMatrix` -> `type: index`, title reworded, a
`## Contents` section (the one body element the `index` archetype requires) and
an explanatory banner added. Closes #74.

**Before** (reproduced in a scratch worktree at `402150362`,
`quire validate spec/matrix/distann.md`, exit 1 — the only failing document in
the whole bundle):

```
line 62:  [TestMatrix] required 'functional_coverage' (table_row(under Functional Requirement Coverage)) is missing
          [TestMatrix] required 'test_cases' (table_row(under Test Case Summary)) is missing
line 328: [TestMatrix] 'non_functional_coverage': table columns
          ["Requirement", "Verification (spec)", "Covering test / evidence", "Status"]
          do not match asserted columns
          ["Non-Functional Req", "Verification Method", "Evidence/Test Cases", "Status"]
```

**After** (at `c2d943730`): `quire validate spec/matrix/distann.md` emits zero
findings for the file, and `quire validate --scope . 'spec/**/*.md'` exits 0
with no line mentioning it. Silence confirmed both ways.

The PR body's duplicate-mint argument also checks out: TM-002's own header says
it supplements TM-001, TC-037..TC-051 are minted by `spec/tests.md`, and the
`test_cases` extraction is id-minting (`id_column: Test ID`), so a
`## Test Case Summary` here would re-mint ids TM-001 owns.

The stated destination — a real nested module matrix, the `filament-ide-rs`
pattern — is now ticketed: **agent-ix/ecaz#76** (open), which restates the
trade and enumerates the three candidate resolutions (nested module matrix; a
new AC-to-evidence-audit archetype; documented exemption).

## What validation the 192 rows had as TestMatrix vs. have as index

The number first: the "192 AC/CON rows" both PR bodies and #76 quote is
TM-002's own Coverage Summary total. It reconciles with the tree: the two
coverage sections carry **185 physical id-keyed rows** (180 keyed on a single
`FR-nnn-AC-n`/`-CON-n`, five requirement-level NFR rows for
NFR-017/018/019/021/022), and one row spans a range (`FR-085-AC-1..AC-8`,
"✅ Covered (8 rows)"), so 185 + 7 = 192 criteria. (The Summary's per-bucket
breakdown does *not* reconcile — that is FND-004.)

**Nominal contract as `TestMatrix`** — what the archetype's `body_extraction`
would enforce on a conforming document (spec-artifacts-process manifest,
`TestMatrix` artifact type), and therefore what #76's implementer must restore
in whatever destination shape:

- Required `## Functional Requirement Coverage` flat table, exact ordered
  columns `Functional Req | Acceptance Criteria | Test Cases | Coverage
  Status`, at least one row.
- Required `## Test Case Summary` table, columns `Test ID | Title | Type |
  Priority | Traces To | Status` (`Priority` presence optional), at least one
  row, with per-row checks:
  - `Test ID` is an **id-minting column**: shape
    `^(TC|IT)(-[A-Za-z0-9]+)*-\d+[A-Za-z0-9]*(-[A-Za-z0-9]+)*$`, registered in
    the corpus, so duplicate ids collide detectably.
  - `Type` closed vocabulary: Unit, Integration, E2E, Property, Fuzz,
    Benchmark, Static, Compile, Snapshot, Manual, Eval.
  - `Priority` closed vocabulary P0–P4 when authored.
  - `Status` leading-marker pattern `^(✅|⚠️|❌|🚧|⛔)(\s+.*)?$`, with class
    semantics (complete/pending/failed/retired) feeding the engine's
    status-lie and undeclared-status reporting.
  - `Traces To` token grammar (`<KIND>-<N>` ids, ranges, `-AC-n` continuation
    and slash shorthands) with **empty cells and `—` deliberately refused** —
    a test tracing to nothing is what the matrix exists to catch.
- Optional-but-asserted-when-present `Stakeholder Requirement Coverage`,
  `User Story Coverage`, and `Non-Functional Requirement Coverage` tables with
  fixed column sets (the last is what TM-002's NFR table failed).
- Frontmatter per `testmatrix-frontmatter.schema.json`; `covers`/`references`
  edges; requirement-id existence checked by the engine's cross-reference
  resolution over extracted rows.

**What the rows actually got as `TestMatrix`: none of the above.** This is the
nuance the seed finding under-stated: extraction failed at the *shape* level
(the three findings quoted above), so the per-row checks — marker vocabulary,
Traces To grammar, id shape, closed Type/Priority vocabularies, reference
resolution — **never executed on these 185 rows**. The per-AC tables
(`AC | Verification (spec) | Covering test / evidence | Status`, nested under
per-FR `###` subsections) match no extraction the archetype declares. What the
retype removed was not row validation — the rows never had it — but the **loud
permanent bundle failure** that acted as a forcing function to give them some.

**Contract as `index`** (spec-artifacts-iso manifest): frontmatter
`index-frontmatter.schema.json` plus one required level-2 `## Contents`
section body. No table extraction, no id minting, no status or trace
vocabulary, no reference resolution over rows. That is the entirety of what
the document is checked for today.

**Residual validation that still exists for DistANN coverage claims:** TM-001's
ten DistANN group rows (TC-037..TC-044, TC-050, TC-051) remain fully
TestMatrix-validated, including their `Traces To` cells, which carry
requirement- and AC-level ids (FR-075..FR-084, NFR-014..NFR-022, StR-008,
FR-076-CON-1, FR-083-AC-4, FR-077-AC/CON ids). AC-level evidence claims — which
test function covers which criterion — exist only in TM-002 and are unchecked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | PR #73 dropped the TC-018 row (untraceable `Future Task 13`) instead of minting the missing HNSW-insert-throughput requirement and repairing the trace. Judgment call, verified mitigated: GAP-004 (`spec/tests.md:489`) tracks the identical work under the same future Task 13. No action needed unless Task 13 is ever scheduled, at which point the requirement and a re-minted TC row come back together. | agent-ix/ecaz#73; spec/tests.md:489 |
| FND-002 | medium | PR #75's disclosed trade is live: TM-002's 192 AC/CON coverage claims are validated by no archetype (silence verified at file and bundle scope). Loud failure was exchanged for silent non-coverage. Destination decision is ticketed as agent-ix/ecaz#76; the "What validation the rows had" section above is the restoration checklist for its implementer. | agent-ix/ecaz#75; agent-ix/ecaz#76; spec/matrix/distann.md |
| FND-003 | medium | The vocabulary #73 retired from TM-001 lives on inside now-unvalidated TM-002: six `🟡 Review-open` rows (FR-083-AC-4..9, `spec/matrix/distann.md:266-271`) and 47 `⚠️` occurrences. `🟡` is not even in TM-002's own Status Legend, and under `type: index` nothing will ever flag any of it. The #73 normalization plus agent-ix/spec-artifacts-process#52 (`⚠️` retirement) covers TM-001 only; TM-002 needs the same sweep when #76 lands it in a validated shape. | agent-ix/ecaz#73; spec/matrix/distann.md:266-271; agent-ix/spec-artifacts-process#52 |
| FND-004 | medium | TM-002's Coverage Summary breakdown is stale against its own rows. Row census at `c2d943730` (185 physical rows, `FR-085-AC-1..AC-8` expanded as 8): ✅ Covered 117, ✅ Bench 6, ⚠️ 47, ❌ 16, 🟡 6 = 192. The Summary claims ✅ 100, ✅ Bench 6, ⚠️ 45, ❌ 41, 🟡 absent. Total matches; every non-Bench bucket is wrong (❌ off by 25). The headline "192" quoted by #75/#76 is the total, which is correct; the breakdown would mislead anyone triaging from it. No archetype checks summary-vs-rows consistency in either type — worth a rule in whatever shape #76 chooses. | spec/matrix/distann.md (Coverage Summary); agent-ix/ecaz#76 |
| FND-005 | low | TM-002's in-file banner says "See agent-ix/ecaz#74" — the symptom ticket #75 closed at merge. The live decision ticket is #76 (filed ~10h after merge). A reader following the banner lands on a closed issue. One-line fix to fold into #76's change. | spec/matrix/distann.md:62; agent-ix/ecaz#74; agent-ix/ecaz#76 |
| FND-006 | low | The document keeps `id: TM-002` and `status: PARTIAL` under `type: index`. The `TM-` identity reads as a matrix mint, and if #76 chooses the nested-module-matrix destination, the natural id for the real matrix is TM-002 — currently claimed by an index document. Reassign or retire the id as part of #76, not before. | spec/matrix/distann.md:2; agent-ix/ecaz#76 |

## Verification log

- Tree: `main` `c2d943730` (fetched and fast-forwarded 2026-08-21); engine:
  `quire 0.23.0` (installed CLI; note it lags the unpublished quire-cli
  v0.28.0 tag).
- `quire validate spec/matrix/distann.md` — zero findings for the file, before
  and within `quire validate --scope . 'spec/**/*.md'` (exit 0; output limited
  to module-load warnings and grammar warnings on other files).
- Same two commands in a scratch worktree at `402150362` (post-#73, pre-#75):
  distann.md fails with exactly the three findings PR #75's body quotes, and is
  the bundle's only failing document — confirming the PR's "after #73: 1
  failure" claim. The "before the status-marker work: 162" figure was not
  re-measured here.
- Marker/row censuses via grep/awk over the two coverage sections; contract
  citations read from `~/.ix/filament/modules/spec-artifacts-process/manifest.yaml`
  and `spec-artifacts-iso/manifest.yaml`; engine behavior from quire-rs
  `src/traceability.rs` and `src/coverage.rs` (read-only).
- This document: `quire validate spec/reviews/gap-analysis-distann-retype.md`
  passes (SpecReview: required Summary section and Findings table with
  FND-ids, low/medium/high severities).
