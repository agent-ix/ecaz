# SPIRE AWS Prep — Local Verification Packet

Task: 30 / Phase 13 (SPIRE AWS Verification)
Sub-phase coverage: 13b.1 (Terraform/scripts already landed), 13c entry-gate
verification, pre-13b execution evidence.
Branch: `task-30-phase13-spire-aws-prep`
Status: in-flight (coder B / SPIRE lane)

## Goal

Stand up a **dedicated PG18 cluster reserved for SPIRE-only verification**,
build SPIRE (IVF + RaBitQ) corpus snapshots at multiple scales, smoke the
multi-disk and multi-node paths, and run the SPIRE AWS `preflight` target
(no AWS spend). Produce filesystem-level cluster snapshots at each corpus
scale so re-runs do not require rebuilding.

This is **prep evidence**, not the AWS run itself. The actual
`make provision` / `pass-correctness` / `pass-representative` runs are a
separate packet that follows this one.

## Scope

Index profile: `ec_spire` (SPIRE wrapping IVF + RaBitQ).
Quantization: `--bits 1` (lowest tier; the page-overflow watch lane).

Corpus scales:

- 10k — pipeline smoke
- 50k — task-50 page-overflow watch
- 100k — multi-disk smoke (local tablespaces)
- 1M  — multi-node smoke (spire-multicluster fixtures)

Smoke surfaces:

- `ecaz dev spire-multicluster smoke-pg18`
- `ecaz dev spire-multicluster customscan-read-pg18`
- `ecaz dev spire-multicluster insert-read-after-customscan-pg18`
- `ecaz dev spire-multicluster transport-overlap-pg18`
- `ecaz dev spire-multicluster fault-pg18`
- `ecaz dev spire-multicluster lifecycle-pg18`

Multi-disk: place SPIRE local store across N tablespaces using the
`local_store_tablespaces` reloption (Phase 12 deliverable).

AWS surface: `make -C infra/spire-aws preflight` only. **No provision,
no instances, no spend.**

## Out of Scope

- Any IVF-only or HNSW baseline (Coder A owns IVF work in a separate
  branch).
- AWS `make provision` / `pass-correctness` / `pass-representative`.
  Those run in the follow-on packet after this prep evidence is
  reviewer-accepted.
- Phase 13a design reviewer signoff and Phase 13d packet acceptance
  (called out as still-open gate items but not produced here).

## Deliverables

- Dedicated SPIRE PG18 cluster on its own port + PGDATA, isolated from
  Coder A's IVF cluster.
- Filesystem snapshots of `$PGDATA` at each corpus scale, written to
  `artifacts/snapshots/<scale>/` (or noted in `artifacts/manifest.md`
  with explicit absolute paths if size precludes packet inclusion).
- Smoke logs for every multicluster fixture above at scales where the
  fixture is meaningful.
- Multi-disk smoke logs.
- `preflight` transcript.
- `artifacts/manifest.md` recording head SHA, dataset identity, command
  used, timestamp, cluster identity (port / PGDATA), and key result
  lines for each artifact.

See `artifacts/manifest.md` for the live ledger as artifacts land.
