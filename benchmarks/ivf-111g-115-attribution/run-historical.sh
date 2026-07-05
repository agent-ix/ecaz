#!/usr/bin/env bash
# Historical per-segment attribution driver. Benches the CONSTANT plain-rabitq
# config (extended sweep) at each lane merge commit so each merge's net effect on
# the stable rabitq path is isolated. NOT a metric sweeper: it only orchestrates
# per-commit worktree + release build/install + the sanctioned `ecaz bench suite`
# runner, with a release-guard before each run.
#
# Run AFTER the HEAD matrix completes (shared PG18 + one installed .so — cannot
# run concurrently). Run from the main checkout:
#   bash benchmarks/ivf-111g-115-attribution/run-historical.sh
set -uo pipefail

MAIN=/home/peter/dev/ecaz
PGCFG=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
BIN=/home/peter/.pgrx/18.3/pgrx-install/bin
SO=/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so
CFG="$MAIN/benchmarks/ivf-111g-115-attribution/configs/constant-rabitq.json"
WT=/home/peter/dev/ecaz-hist
export PGHOST=/home/peter/.pgrx PGPORT=28818 PGDATABASE=ivf_attr_bench

# commit -> label (baseline first)
COMMITS=(
  "99dc70e53:baseline"
  "61fd84f95:111g"
  "6d60eec50:112"
  "9ddb3be7c:113"
)

for entry in "${COMMITS[@]}"; do
  sha="${entry%%:*}"; label="${entry##*:}"
  echo "===== HISTORICAL $label ($sha) ====="
  # fresh worktree at the commit (detached)
  git -C "$MAIN" worktree remove --force "$WT" 2>/dev/null || true
  rm -rf "$WT"
  git -C "$MAIN" worktree add --detach "$WT" "$sha" || { echo "!! worktree add failed $label"; continue; }
  # build + install that commit's release .so and CLI from the worktree
  ( cd "$WT" && cargo pgrx install --release --no-default-features --features pg18 --pg-config "$PGCFG" ) \
    || { echo "!! pgrx install failed $label"; continue; }
  ( cd "$WT" && cargo build --release -p ecaz-cli ) || { echo "!! cli build failed $label"; continue; }
  ECAZ="$WT/target/release/ecaz"
  # fresh DB with this commit's SQL
  $BIN/dropdb --if-exists ivf_attr_bench
  $BIN/createdb ivf_attr_bench
  $BIN/psql -d ivf_attr_bench -c "CREATE EXTENSION ecaz;" >/dev/null 2>&1
  prof=$($BIN/psql -d ivf_attr_bench -tAc 'select ecaz_build_profile();' 2>/dev/null)
  isha=$(sha256sum "$SO" | cut -d' ' -f1)
  echo "  installed: profile=$prof sha=${isha:0:12}"
  if [ "$prof" != "release" ]; then echo "!! not release ($prof) for $label — skipping"; continue; fi
  # run the constant config from MAIN (so data/staged-current + config resolve),
  # using THIS commit's ecaz binary + installed .so.
  ( cd "$MAIN" && "$ECAZ" bench suite run --config "$CFG" \
      --artifact-dir "$MAIN/benchmarks/ivf-111g-115-attribution/artifacts/hist-$label" ) \
    && echo "===== OK historical $label =====" || echo "===== FAILED historical $label ====="
done
# cleanup worktree, restore HEAD release .so
git -C "$MAIN" worktree remove --force "$WT" 2>/dev/null || true
rm -rf "$WT"
echo "===== HISTORICAL LAYER COMPLETE — reinstall HEAD (fixed) release .so next ====="
