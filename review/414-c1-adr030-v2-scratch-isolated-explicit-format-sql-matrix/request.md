# Review Request: C1 ADR-030 V2 Scratch Isolated Explicit-Format SQL Matrix

Current head at execution: `4857aa6`

## Context

Packet `405` made the task-15 landing-proof claim, but reviewer feedback on
that packet correctly called out two remaining proof gaps:

1. the landing proof had been gathered on `~/.pgrx`, not the hardened scratch
   cluster
2. the branch still lacked a clean planner-facing SQL latency matrix alongside
   the direct-runtime recall matrix

Packet `413` addressed the second problem only halfway:

- it reran the full direct-runtime matrix on the scratch cluster
- it explicitly recorded why a shared-table verified SQL matrix was not honest:
  the planner cross-chose between sibling `m=8` and `m=16` indexes on the same
  corpus table

This packet closes that remaining SQL-side proof gap without lying to the
planner. It does so by creating isolated one-index-per-table explicit-format
surfaces on the scratch cluster and then running the planner-verified warm SQL
launcher against those surfaces only.

## Problem

The branch already had the clean direct-runtime read:

- `10k`, `50k`
- `turboquant`, `pq_fastscan`
- `m=8`, `m=16`
- `ef_search=40,64,100,128,160,200`

What it still did not have was the matching planner-facing SQL read on a clean
surface. The shared canonical tables are not that surface right now because the
planner legitimately prefers the sibling index it thinks is cheaper.

So the honest question became:

> On scratch only, with one explicit-format index per table, what does the warm
> verified SQL spectrum look like for the same `10k/50k × turboquant/pq_fastscan
> × m=8/16 × ef=40..200` matrix?

## Planned Slice

No code change.

This packet only rebuilds the current-head scratch lane, creates isolated
explicit-format benchmark surfaces, runs the verified SQL launcher, and records
the result.

## Environment

### Scratch-cluster provenance

The repo-local scratch pgrx home had been cleaned out, so I first restored the
minimum config needed for the approved scratch wrappers:

- `install -D /home/peter/.pgrx/config.toml /tmp/tqvector_pgrx_home/config.toml`

Then I rebuilt and restarted the current-head scratch lane:

- `./scripts/install_adr030_pg17_pg_test.sh`
- `./scripts/restart_adr030_scratch.sh --window 64 --grouped-score-mode binary`
- `./scripts/pg17_scratch_psql.sh --sql "CREATE EXTENSION IF NOT EXISTS tqvector CASCADE;"`

Backend-visible runtime settings from scratch:

| grouped_build_enabled | grouped_scan_enabled | grouped_scan_window | grouped_scan_score_mode | grouped_scan_rerank_mode | grouped_scan_rerank_source_column | grouped_exact_traversal_enabled |
|---|---|---|---|---|---|---|
| `t` | `t` | `64` | `binary` | `heap_f32` | `build_source_column` | `f` |

So every SQL run below is on the same scratch lane as packet `413`:

- scratch cluster only (`/tmp/tqvector_pgrx_home`)
- `pq_fastscan` traversal score mode: `binary`
- `pq_fastscan` rerank mode: `heap_f32`
- rerank source column: `build_source_column`
- exact traversal disabled

### Git provenance

Head at execution:

- `4857aa6 Add review packet for explicit-format runtime matrix`

The current checkout was **not** globally clean. `git status --porcelain`
showed unrelated local dirt under `plan/`, `vendor/`, `spec/`, `.codex`,
`review/*/feedback/`, and `tmp/`.

What matters for this packet is narrower:

- `git status --porcelain -- src scripts docs` returned nothing

So there were no local uncommitted modifications under the runtime code or
benchmark scripts while these measurements ran.

## Isolated SQL Surfaces

Each surface below was loaded with the existing scratch wrapper using the
canonical staged TSVs plus `--allow-manifest-mismatch`, one index per table:

- `tqhnsw_real_10k_turboquant_m8only`
- `tqhnsw_real_10k_turboquant_m16only`
- `tqhnsw_real_10k_pq_fastscan_m8only`
- `tqhnsw_real_10k_pq_fastscan_m16only`
- `tqhnsw_real_50k_turboquant_m8only`
- `tqhnsw_real_50k_turboquant_m16only`
- `tqhnsw_real_50k_pq_fastscan_m8only`
- `tqhnsw_real_50k_pq_fastscan_m16only`

Each verified SQL sweep then used the same warm timing contract:

- `--query-limit 50`
- `--cache-state warm-after-prime3`
- `--warmup-passes 3`
- `--session-mode per-cell`
- `--timing-mode cached-plan`

The verified launcher checked the expected index at every measured cell and
aborted if the planner drifted.

## Results

All numbers below are the planner-verified **mean SQL latency in ms** from the
summary files under `/tmp/adr030_sql_*.summary`.

### 10k, `m=8`

Artifacts:

- `/tmp/adr030_sql_tqhnsw_real_10k_turboquant_m8only.summary`
- `/tmp/adr030_sql_tqhnsw_real_10k_pq_fastscan_m8only.summary`

| ef_search | turboquant mean ms | pq_fastscan mean ms |
|----------:|-------------------:|--------------------:|
| 40  | `0.949` | `1.714` |
| 64  | `1.209` | `2.508` |
| 100 | `1.631` | `2.938` |
| 128 | `1.941` | `3.122` |
| 160 | `2.231` | `3.275` |
| 200 | `2.558` | `3.435` |

### 10k, `m=16`

Artifacts:

- `/tmp/adr030_sql_tqhnsw_real_10k_turboquant_m16only.summary`
- `/tmp/adr030_sql_tqhnsw_real_10k_pq_fastscan_m16only.summary`

| ef_search | turboquant mean ms | pq_fastscan mean ms |
|----------:|-------------------:|--------------------:|
| 40  | `0.985` | `1.650` |
| 64  | `1.333` | `2.491` |
| 100 | `1.721` | `2.992` |
| 128 | `2.079` | `3.176` |
| 160 | `2.431` | `3.364` |
| 200 | `2.770` | `3.450` |

### 50k, `m=8`

Artifacts:

- `/tmp/adr030_sql_tqhnsw_real_50k_turboquant_m8only.summary`
- `/tmp/adr030_sql_tqhnsw_real_50k_pq_fastscan_m8only.summary`

| ef_search | turboquant mean ms | pq_fastscan mean ms |
|----------:|-------------------:|--------------------:|
| 40  | `1.390` | `1.898` |
| 64  | `1.819` | `2.812` |
| 100 | `2.504` | `3.300` |
| 128 | `2.972` | `3.540` |
| 160 | `3.472` | `3.706` |
| 200 | `4.148` | `4.038` |

### 50k, `m=16`

Artifacts:

- `/tmp/adr030_sql_tqhnsw_real_50k_turboquant_m16only.summary`
- `/tmp/adr030_sql_tqhnsw_real_50k_pq_fastscan_m16only.summary`

| ef_search | turboquant mean ms | pq_fastscan mean ms |
|----------:|-------------------:|--------------------:|
| 40  | `1.857` | `2.104` |
| 64  | `2.612` | `3.019` |
| 100 | `3.708` | `3.921` |
| 128 | `4.437` | `4.263` |
| 160 | `5.194` | `4.531` |
| 200 | `6.054` | `5.264` |

## Readout

### 1. The SQL matrix is now clean and planner-honest

This packet fully closes the gap packet `413` left open:

- every measured SQL cell ran on the scratch cluster
- every measured SQL cell had exactly one target index on its table
- every measured SQL cell was verified by plan inspection before timing

So this is the first honest full planner-facing SQL spectrum for the explicit
format families on the current branch head.

### 2. `10k` still favors `turboquant` on SQL latency

On both `10k` lanes, `turboquant` is faster at every measured `ef_search`:

- `10k, m=8, ef=128`: `1.941ms` vs `3.122ms`
- `10k, m=16, ef=128`: `2.079ms` vs `3.176ms`

Paired with packet `413`, that means `pq_fastscan` is buying higher recall on
`10k`, but it is paying for that recall in planner-facing SQL latency.

### 3. `50k, m=8` is still mostly `turboquant`-faster, but the gap closes

At `50k, m=8`, `turboquant` stays ahead through `ef=160`, but the SQL gap
narrows sharply and even inverts slightly by `ef=200`:

- `ef=128`: `2.972ms` vs `3.540ms`
- `ef=160`: `3.472ms` vs `3.706ms`
- `ef=200`: `4.148ms` vs `4.038ms`

That does not overturn the general `m=8` read, but it does say the SQL surface
is not a simple “always slower” story once `50k` and higher `ef` enter.

### 4. The serious `50k, m=16` operating area now crosses over in `pq_fastscan`'s favor

This is the most important result in the packet.

On the clean isolated SQL surface:

- `50k, m=16, ef=40`: `turboquant` faster (`1.857ms` vs `2.104ms`)
- `50k, m=16, ef=64`: `turboquant` faster (`2.612ms` vs `3.019ms`)
- `50k, m=16, ef=100`: `turboquant` faster (`3.708ms` vs `3.921ms`)
- `50k, m=16, ef=128`: `pq_fastscan` faster (`4.263ms` vs `4.437ms`)
- `50k, m=16, ef=160`: `pq_fastscan` faster (`4.531ms` vs `5.194ms`)
- `50k, m=16, ef=200`: `pq_fastscan` faster (`5.264ms` vs `6.054ms`)

That crossover matters because packet `413` already showed `pq_fastscan` ahead
on recall at those same serious operating points:

- `50k, m=16, ef=128`: `0.9635` vs `0.9342`
- `50k, m=16, ef=160`: `0.9657` vs `0.9366`
- `50k, m=16, ef=200`: `0.9671` vs `0.9376`

So the current branch read is now stronger than “`pq_fastscan` wins recall but
loses latency.”

On the clean isolated `50k, m=16, ef>=128` planner-facing surface,
`pq_fastscan` is:

- higher recall, per packet `413`
- lower mean SQL latency, per this packet

### 5. This is still not the canonical shared-table planner verdict

These SQL numbers are intentionally isolated one-index-per-table surfaces.
That is exactly what makes them honest today, but it is also why they should
not be misreported as the canonical shared-table planner read.

The shared-table planner problem remains what packet `413` said it was:

- the planner cross-chooses between sibling `m=8` and `m=16` indexes
- a shared-table SQL spectrum therefore still needs either planner work or a
  different measurement surface

What this packet does prove is narrower and still valuable:

- when the planner is given a clean explicit-format target surface,
  `pq_fastscan` is not intrinsically disqualified on SQL latency
- in the branch’s most serious `50k, m=16, high-ef` area, it is actually ahead

## Validation

No code changed in this packet. I still reran the branch validation commands on
the current workspace so the closeout is explicit about the test-execution
state:

- `cargo test`
  - **fails**
  - same unresolved PostgreSQL symbol family as before:
    - `CurrentMemoryContext`
    - `PG_exception_stack`
    - `error_context_stack`
    - `CopyErrorData`
    - `errstart`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`
  - **fails**
  - same unresolved PostgreSQL/pgrx linker boundary as `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - **passes**

That means the reviewer’s “say the pg-test gap out loud” concern is still real.
The branch now has clean metrics, but it still does **not** have a successful
local `cargo test` / `cargo pgrx test pg17` execution on this workstation.

This also means packet `393`’s round-trip tests remain defined but unexecuted
locally here:

- `test_tqhnsw_turboquant_reloption_round_trip`
- `test_tqhnsw_pq_fastscan_reloption_round_trip`

## Outcome

This packet materially improves the branch’s merge-readiness evidence:

1. packet `405`’s scratch-cluster provenance concern is now closed
2. packet `413`’s missing SQL-spectrum gap is now closed
3. the clean planner-facing SQL story is now:
   - `10k`: `turboquant` faster
   - `50k, m=8`: mostly `turboquant` faster, near parity by `ef=200`
   - `50k, m=16`: `pq_fastscan` faster from `ef=128` onward
4. paired with packet `413`, the serious current-head `50k, m=16, ef>=128`
   area is now favorable to `pq_fastscan` on both recall and SQL latency
5. the remaining branch risk is no longer “dirty metrics” or “unknown planner
   SQL behavior on a clean target surface”; it is the still-unresolved local
   pg-test linker boundary

## Next Slice

If the goal is “ready to land on `main`,” the remaining work is not another
runtime benchmark. It is one of:

1. resolve the PostgreSQL/pgrx linker boundary and run the outstanding test
   surface for real, or
2. make an explicit merge decision that the metrics/proof bar is satisfied and
   the remaining unexecuted pg-test surface is an external infra limitation

Either way, the branch no longer needs more measurement cleanup.
