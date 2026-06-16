# Task 109 — Standardized ecaz bench sweep config (config + docs only)

Branch: `task-108-109-comparator-unification`
Head SHA at request: `56ce92879`
Packet: `reviews/task-109/001-standard-sweep-config/`

## What changed

Promote the four per-lane configs under `crates/ecaz-cli/suites/current/` into
**THE standard ecaz sweep**, one canonical config per supported host, and
document the "run-as-is, justify custom" convention. No runner code changed
(`ecaz bench suite`, FR-038, is unchanged).

### Canonical lane configs (now full standard matrix)

`m5-local.json`, `intel-local.json`, `aws-intel.json`, `aws-graviton.json` each
now cover the full standard matrix — the standard access-method profiles
(`ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`) × the standard scales
(**10k / 50k / 100k / 1m**) × the standard `load` / `recall` / `latency` /
`storage` steps (**65 steps each**: precheck + 4×4×4) — replacing the prior
hand-curated 1–2-profile, 100k-only subsets. recall/latency `sweep` is set to
each profile's `default_sweep` from `crates/ecaz-cli/src/profiles.rs` verbatim
(suite steps require an explicit `sweep`, so the standard is to copy the
registered default):

- `ec_hnsw` → `[40,64,100,128,160,200]`
- `ec_diskann` → `[64,128,200,400,800]`
- `ec_ivf` → `[8,16,24,32,48,64]`
- `ec_spire` → `[8,16,24,32]`

The corpus is profile-agnostic (one `ec_real_100k` TSV serves all four access
methods via separate `--profile` loads into per-profile prefixes), so the full
4-profile matrix uses the corpus each lane already stages — no new corpus
needed. Competitor numbers come from `comparator` steps (Task 108), never from
re-running this sweep.

### Docs + convention

- `CLAUDE.md` (Benchmark Runner section): new "The Standard ecaz Sweep"
  subsection — names the four canonical configs, the profile/step/sweep
  standard, and the convention: **run the standard lane config as-is; only
  hand-author a bespoke SuiteConfig for a non-standard grid/scale/option and
  state the reason in the packet `manifest.md`**. Subset with
  `--only-tag ec_real_100k` / `--only-tag hnsw` when resource-constrained
  instead of editing the config.
- `crates/ecaz-cli/README.md`: mirrors the same standard + convention above the
  "Current benchmark lanes" runbook.

## Corpus staging convention

Each lane reads a single per-environment staged dir, named
`ec_real_{10k,50k,100k,1m}_{corpus,queries}.tsv` + `_manifest.json`:

- local lanes (`m5-local`, `intel-local`): `data/staged-current/`
- AWS lanes (`aws-intel`, `aws-graviton`): `/var/lib/pgsql/18/datasets/staged-current/`

Staging the four scales there is the run prerequisite (the corpus exists — it's
been exercised on these hosts; the config is the recipe pointing at the
canonical location).

## Commit

- `56ce92879` (100k cut, superseded) → full 4-scale matrix in the follow-up
  commit on this branch.

## Verification

- `ecaz bench suite run --config crates/ecaz-cli/suites/current/<lane>.json
  --dry-run` expands cleanly for all four lanes — **65 steps each** (precheck +
  4 scales × 4 profiles × 4 steps), all four scales (10k/50k/100k/1m) present,
  each recall/latency step carrying the profile `default_sweep`, no errors.
  (Local `audit` will flag missing corpus inputs since the TSVs live on each
  lane's host, not on this machine; the dry-run is the structural check.)

## Follow-up (noted, not this cut)

- Auditing/retiring the redundant per-task suites under `crates/ecaz-cli/suites/`
  remains a follow-up.
