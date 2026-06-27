# Task 111g — Packet 002: Rerank-representation benchmark gate (Phase 3)

Status: **open — BLOCKED on benchmark prerequisites in this environment.**

Covers Task 111g Phase 3 (the benchmark gate). The code under review is in
packet 001. This packet delivers the **committed, validated `ecaz bench suite`
config** for the rerank-representation matrix + matched-recall baseline, plus
the exact run recipe and the promote/iterate decision framework — but the run
itself could not be executed here (see Blocker).

## What this packet contains

- `artifacts/task111g-rerank-rep-suite.json` — the bespoke SuiteConfig (justified
  below) for:
  - **coarse_rerank × rerank_format ∈ {f32, f16, rabitq4}** at **50k and 100k**,
    each with `load` + `recall` + `latency` + `storage` (the standard 4 steps),
    `nprobe` sweep `[8,16,24,32,48,64]` (the registered ec_ivf default).
  - **matched-recall baselines**: `dense-rb8` (`storage_format=rabitq`,
    `quant_bits=8`, `rerank=off`) and `row-f32` (`storage_format=rabitq`,
    `quant_bits=1` dense + `rerank=heap_f32`, `rerank_width=200`), same scales /
    steps / sweep. The matched-recall comparison (latency-at-recall≈0.97 and
    ≈0.99) is derived **post-hoc** from the recall+latency sweeps (pick the
    nprobe nearest each recall target per variant), so no new suite step type is
    required.
- `artifacts/suite-audit.log` — `ecaz bench suite audit` output: the config
  parses and every step is structurally valid; the only issues are the 30
  missing staged-corpus references (expected — corpora are not staged in this
  sandbox).
- `artifacts/suite-dry-run.log` — `ecaz bench suite run --dry-run` output:
  confirms the reloption passthrough expands correctly, e.g.
  `--storage-format coarse_rerank --reloption ... --reloption rerank_format=f16`,
  the dense-rb8 baseline (`quant_bits=8 rerank=off`), and the row-f32 baseline
  (`quant_bits=1 dense + rerank=heap_f32 rerank_width=200`).

## Why a bespoke config (not the canonical lane config)

The standard lane sweep (`crates/ecaz-cli/suites/current/<lane>.json`) carries
the 4-profile × 4-scale × 4-step matrix but does **not** carry a
coarse_rerank × rerank_format axis. Task 111g genuinely needs that non-standard
grid (the rerank-representation matrix + the two matched-recall baselines), so
this packet hand-authors the config per the CLAUDE.md exception and states the
reason here. Everything else follows the standard: `ec_ivf` profile, the
registered default nprobe sweep, the standard load/recall/latency/storage steps,
k=10.

## Supporting CLI change (committed in packet 001's branch)

`crates/ecaz-cli/src/profiles.rs` — added `coarse_format`, `coarse_bits`,
`rerank_placement`, `rerank_format` to `EC_IVF.known_reloptions` so the suite
runner does not warn on the coarse_rerank reloption passthrough. `ecaz-cli`
builds clean; `cargo test -p ecaz-cli profiles` passes (25 tests).

## BLOCKER — why the run is not in this packet

The benchmark could not be executed in this agent sandbox because:

1. **No staged corpora.** `data/staged-current/ec_real_{50k,100k}_*` are not
   present (the standard run prerequisite is staging the four scales there).
2. **No installed branch extension.** The PG18 instance does not have the
   111g `ecaz.so` (with coarse_rerank f16/rabitq4 rerank) installed; the only
   `ecaz` binary in this worktree is the CLI, and the extension would need
   `cargo pgrx install`/`ecaz dev install` from this branch first.
3. The matched-recall framing requires the recall+latency sweeps to actually
   run before the post-hoc analysis can be done.

None of these are code defects; they are environment provisioning that this
sandbox does not have. Per NFR-007 / the benchmark-provenance rule I am **not**
fabricating numbers — the config is committed and dry-run/audit-validated so a
provisioned lane can run it as-is.

## Run recipe (for a provisioned local Intel / AWS lane)

1. Build + install the branch extension on the PG18 instance
   (`cargo pgrx install --no-default-features --features pg18` or
   `ecaz dev install`); verify the installed `.so` is the release build.
2. Stage real 50k + 100k corpora at `data/staged-current/` (local) or
   `/var/lib/pgsql/18/datasets/staged-current/` (AWS), named
   `ec_real_{50k,100k}_{corpus,queries}.tsv` + `_manifest.json`.
3. `ecaz bench suite audit --config reviews/task-111g/002-rerank-rep-benchmark-gate/artifacts/task111g-rerank-rep-suite.json`
   (expect 0 issues once corpora are staged).
4. `ecaz bench suite run --config reviews/task-111g/002-rerank-rep-benchmark-gate/artifacts/task111g-rerank-rep-suite.json --artifact-dir reviews/task-111g/002-rerank-rep-benchmark-gate/artifacts`
5. Commit `suite-manifest*.json` + `suite-results*.jsonl` + the cited
   recall/latency/storage logs into this packet's `artifacts/`, update
   `manifest.md` with the head SHA / lane / key result lines, and add the
   promote/iterate recommendation below.

## Promote / iterate decision framework (to fill after the run)

- **Promote f16** if it holds f32 recall (Δrecall ≤ ~0.002 at every nprobe) at
  ≤ f32 latency and ~half the rerank bytes (packet 005 showed f16 = f32 recall
  at half bytes on the SQL sidecar; this confirms it through the in-AM path).
- **Promote rabitq4** if it keeps recall within the high-recall target band
  (≈0.97/0.99) at materially lower latency/bytes than f16 — i.e. a compact
  rerank that *keeps* f32 recall (the open question from 005, where rabitq8's
  0.946 was too lossy).
- **Iterate / reject** any representation that drops below the ≈0.97 band at the
  matched nprobe, and report it against the dense-rb8 / row-f32 baselines:
  state, at recall ≈0.97 and ≈0.99, the p50 latency of coarse_rerank-{f32,f16,
  rabitq4} vs dense-rb8 vs row-f32.

## Review ask

Please gate whether (a) this committed config + recipe is an acceptable Phase 3
deliverable given the sandbox cannot run it, or (b) you want me to run it on a
provisioned lane (I'd need staged corpora + the branch extension installed), and
whether the matched-recall post-hoc framing is sufficient or you want a dedicated
matched-recall suite step type added to the runner first.
