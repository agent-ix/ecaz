# Review Request: First-class `ecaz corpus fetch` for the canonical real corpus

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/Cargo.toml`
- `crates/ecaz-cli/src/commands/corpus/mod.rs`
- `crates/ecaz-cli/src/commands/corpus/fetch.rs`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/README.md`

## What this packet is

Task 17 still needs a clean DiskANN closeout path on pg18. At this point the
remaining blocker is not another AM-side change: it is that `ecaz-cli`, the
canonical operator surface, could prepare and load a local parquet release but
still had no first-class way to fetch that release in the first place.

This packet adds a narrow `ecaz corpus fetch` command to `ecaz-cli`. The change
is intentionally generic and cherry-pickable to `main`: it does not add any
task-17-only special case, and it stays confined to the corpus CLI surface plus
README/help wiring.

## What changed

### `crates/ecaz-cli/src/commands/corpus/fetch.rs`

- New first-class fetch command:

```rust
#[derive(Args, Debug)]
pub struct FetchArgs {
    #[arg(long, default_value = DEFAULT_DATASET)]
    pub dataset: String,
    #[arg(long)]
    pub output_dir: PathBuf,
    #[arg(long)]
    pub revision: Option<String>,
    #[arg(long, default_value_t = false)]
    pub force: bool,
}
```

- Added a pinned remote dataset registry for the canonical real-corpus source:
  Qdrant's DBpedia OpenAI `text-embedding-3-large` `1536`-dimensional 1M-row
  dataset on Hugging Face.

- The fetch logic is explicit and deterministic:
  - fixed dataset name / repo / default revision
  - deterministic shard names (`train-00000-of-00026.parquet` …)
  - resolve URLs under the Hugging Face dataset `resolve/` path
  - output layout under `<output-dir>/data/`
  - atomic `.part` download then rename
  - `--force` to overwrite existing shards; otherwise existing files are skipped

- The command also writes a small local metadata file:

```rust
pub const FETCH_MANIFEST_FILE: &str = "ecaz_fetch_manifest.json";
```

with dataset name, source label, repo, revision, parquet directory, shard list,
and fetch timestamp.

- Pure unit-test seams pin:
  - dataset resolution
  - shard filename generation
  - download URL shape
  - fetch-manifest contents
  - temp download path shape

### `crates/ecaz-cli/src/commands/corpus/mod.rs`

- Added the new subcommand to the corpus tree:

```rust
pub enum CorpusCommand {
    Fetch(FetchArgs),
    Load(LoadArgs),
    Inspect(InspectArgs),
    List,
    Generate(GenerateArgs),
    Prepare(PrepareArgs),
}
```

- Routing stays narrow: `Fetch` only calls the new fetch module.

### `crates/ecaz-cli/src/cli.rs`

- Added a parser test covering the new invocation shape:

```rust
ecaz corpus fetch \
  --dataset dbpedia-openai3-large-1536-1m \
  --output-dir /data/real-corpus \
  --revision main \
  --force
```

This keeps the fetch surface pinned at the clap layer instead of leaving it as
README-only behavior.

### `crates/ecaz-cli/Cargo.toml`

- Added `reqwest` with `rustls-tls` + `stream` to give the CLI a native async
HTTP download path instead of relying on external wrappers or env-sensitive
shell tooling.

### `crates/ecaz-cli/README.md`

- Added `corpus fetch` to the command tree.
- Documented the intended real-corpus flow:
  1. `ecaz corpus fetch`
  2. `ecaz corpus prepare`
  3. `ecaz corpus load --profile ec_diskann`

That makes the canonical path explicit in the CLI docs instead of expecting the
operator to know a separate script-only fetch step.

## Why this slice

- This is the narrow tooling change that is actually blocking DiskANN closeout:
  the operator surface could not fetch the canonical real corpus for itself.
- The implementation is generic enough to land on `main` immediately:
  one command, one remote dataset registry, no task-17-specific branching.
- It avoids the wrong alternatives:
  - no env-var fetch wrappers
  - no shelling out to external dataset tools
  - no more `scripts/` drift

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 216 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also ran locally on `pg18` for this slice:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- A live network fetch smoke against Hugging Face. This slice keeps the code
surface main-ready and locally verified; it does not claim a successful remote
download artifact.
- Chaining `fetch` directly into `prepare` or `load`. Keeping the steps explicit
preserves the current reviewable pipeline and avoids a larger orchestration
change.
- Adding arbitrary remote-dataset support. The current need is one canonical
first-class dataset, not a generalized remote package manager.
