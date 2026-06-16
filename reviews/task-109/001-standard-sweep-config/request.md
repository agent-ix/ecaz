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
now cover the standard access-method profiles (`ec_hnsw`, `ec_ivf`,
`ec_diskann`, `ec_spire`) × the standard `load` / `recall` / `latency` /
`storage` steps at the scale each lane stages today (100k), replacing the prior
hand-curated 1–2-profile subsets. recall/latency `sweep` is set to each
profile's `default_sweep` from `crates/ecaz-cli/src/profiles.rs` verbatim
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
  state the reason in the packet `manifest.md`**. Adding a scale = stage its
  TSVs on the host and append the load/recall/latency/storage quartet per
  profile.
- `crates/ecaz-cli/README.md`: mirrors the same standard + convention above the
  "Current benchmark lanes" runbook.

## Commit

- `56ce92879` Task 109: canonical per-lane standard sweep configs + convention

## Verification

- `ecaz bench suite run --config crates/ecaz-cli/suites/current/<lane>.json
  --dry-run` expands cleanly for all four lanes — 17 steps each (precheck + 4
  profiles × 4 steps), each recall/latency step carrying the profile
  `default_sweep`. (Local `audit` will flag missing corpus inputs for the AWS
  lanes since their TSVs live on the remote host, not locally; the dry-run is
  the structural check.)

## Scope note / follow-up

- Lanes cover **100k** today (the scale staged on every host). Extending each
  lane to 10k/50k/1m is the documented fill-in path: stage the TSVs on the host,
  append the per-profile quartet at that scale with `default_sweep`. Not done in
  this cut because the 50k/1m corpus TSVs are not staged on every lane's host
  (the AWS snapshot holds them as loaded tables, not local TSVs).
- Auditing/retiring the redundant per-task suites under `crates/ecaz-cli/suites/`
  remains a noted follow-up (not this cut).
