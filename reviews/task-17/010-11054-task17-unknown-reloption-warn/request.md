# Review Request: `ecaz corpus load` — unknown-reloption-key warning

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/profiles.rs`
- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

Operator-UX slice inside `ecaz-cli`. Adds a pre-flight check that warns
(does not error) when a `--reloption key=value` uses a key outside the
active profile's `known_reloptions` set, so typos land on the operator's
stderr seconds into the loader run instead of surfacing as an opaque
`unrecognized parameter` error from `CREATE INDEX` after the corpus is
already loaded.

The motivation is DiskANN: `ec_diskann` has six `known_reloptions`
(`graph_degree`, `build_list_size`, `list_size`, `rerank_budget`,
`top_k`, `alpha`) plus `storage_format`, and none of them are surfaces
operators already carry in muscle memory from HNSW.
`--reloption graph_degre=48` should not require a corpus reload to
diagnose.

## What changed

### `crates/ecaz-cli/src/profiles.rs`

- New method `IndexProfile::unknown_reloption_keys(&self,
  &[(String, String)]) -> Vec<&str>` that returns the subset of passed
  reloption keys not present in `self.known_reloptions`.
- Four unit tests:
  - A DiskANN sample where two typos (`graph_degre`, `rerank_budge`)
    are returned in order alongside known keys (`graph_degree`,
    `alpha`), which get filtered out.
  - Empty result when all keys are known (HNSW `m` + `ef_construction`).
  - Empty result when no reloptions were supplied.
  - Case-sensitivity guarded: `GRAPH_DEGREE` is returned as unknown so
    the operator sees a clean warning rather than a silent downcase.
    pg_class.reloptions stores canonical lowercase, so matching case
    here is the right default.

### `crates/ecaz-cli/src/commands/corpus/load.rs`

Right after profile resolution and the `--m`-on-non-HNSW guard, a new
six-line block emits one `[loader] warning:` line listing the unknown
keys and the full `known_reloptions` set for the active profile. The
warning is advisory only — the keys still flow through to the existing
`plan_index_jobs` / `build_create_index_sql` path exactly as before.
This matches the intent documented in the `known_reloptions` doc
comment on `IndexProfile`: "Unknown keys are still accepted by
`--reloption` passthrough — this set is for help text only."

The message:

```
[loader] warning: profile "ec_diskann" does not list graph_degre, rerank_budge
as known reloptions; passing through verbatim. Known reloptions:
graph_degree, build_list_size, list_size, rerank_budget, top_k, alpha,
storage_format
```

(Singular / plural chosen based on `unknown.len() == 1`.)

## Why this slice

- Fully inside `crates/ecaz-cli/`: no `scripts/*` churn, no overlap with
  the native-build / deletion lane the other agent owns on `main`.
- Benefits DiskANN operators disproportionately (six new reloption
  names) without regressing HNSW (passthrough + case-sensitive match
  keep the existing `build_source_column` / `storage_format` path).
- Warning-not-error keeps room for future AMs whose reloption set the
  CLI doesn't know about yet. Consistent with the `IndexProfile` docs.
- Small enough to land as its own commit with unit tests only (no DB
  integration needed) and leaves the surrounding `run` control flow
  intact.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3
test result: ok. 175 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.01s
```

Up from 171 before this packet — the four new tests cover the typo
path, the all-known path, the empty-input path, and case-sensitivity.

## Follow-ups intentionally not in this packet

- Nearest-match suggestion ("did you mean `graph_degree`?" via
  Levenshtein-1). Worth doing once we have more profiles; premature
  today because both HNSW and DiskANN known sets fit on one line.
- Applying the same check to `--reloption` on other commands — only
  `corpus load` currently accepts `--reloption`, so no further sites
  to wire.
- Promoting the warning to an error behind a strict flag. Not needed
  yet; the profile docs explicitly promise passthrough.
