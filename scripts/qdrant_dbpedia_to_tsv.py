#!/usr/bin/env python3
"""Convert the Qdrant DBpedia parquet release into canonical TSV fixtures.

This script implements the reproducible subset-selection contract documented in
``docs/RECALL_REAL_CORPUS.md``:

- sort the full parquet release by `id` ascending
- take the first N rows as the corpus subset
- take the next M rows as the query subset
- emit canonical `<prefix>_{corpus,queries}.tsv` files plus
  `<prefix>_manifest.json`

The script requires `pyarrow` to be installed locally. The repository does not
vendor parquet dependencies or dataset binaries.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import sys
from typing import Iterable, Sequence


DEFAULT_SOURCE_DATASET = "Qdrant dbpedia-entities-openai-1M"
DEFAULT_DIM = 1536
DEFAULT_ID_COLUMN = "id"
VECTOR_COLUMN_CANDIDATES = (
    "embedding",
    "vector",
    "values",
    "openai",
    "text_embedding",
)


@dataclass(frozen=True)
class SubsetProfile:
    prefix: str
    corpus_rows: int
    query_rows: int

    @property
    def query_start(self) -> int:
        return self.corpus_rows


PROFILES: dict[str, SubsetProfile] = {
    "tqhnsw_real_50k": SubsetProfile(
        prefix="tqhnsw_real_50k",
        corpus_rows=50_000,
        query_rows=1_000,
    ),
    "tqhnsw_real_10k": SubsetProfile(
        prefix="tqhnsw_real_10k",
        corpus_rows=10_000,
        query_rows=200,
    ),
}


@dataclass
class FileManifest:
    file: str
    rows: int
    sha256: str
    first_id: int | None
    last_id: int | None


def _import_pyarrow():
    try:
        import pyarrow.dataset as ds
    except ModuleNotFoundError as exc:
        raise SystemExit(
            "pyarrow is required for parquet conversion. Install it locally before "
            "running scripts/qdrant_dbpedia_to_tsv.py."
        ) from exc
    return ds


def _canonical_float(value: float) -> str:
    if not math.isfinite(value):
        raise ValueError(f"non-finite value {value!r} is not allowed in embeddings")
    return format(float(value), ".9g")


def _canonical_json_array(values: Sequence[float]) -> str:
    return "[" + ",".join(_canonical_float(value) for value in values) + "]"


def _coerce_vector(value, *, dim: int, row_id: int, vector_column: str) -> list[float]:
    if value is None:
        raise ValueError(f"row id {row_id}: vector column {vector_column!r} is null")
    if hasattr(value, "to_pylist"):
        value = value.to_pylist()
    elif not isinstance(value, list):
        value = list(value)
    floats = [float(v) for v in value]
    if len(floats) != dim:
        raise ValueError(
            f"row id {row_id}: expected dim {dim} in column {vector_column!r}, got {len(floats)}"
        )
    return floats


def _resolve_vector_column(schema_names: Sequence[str], requested: str | None) -> str:
    if requested:
        if requested not in schema_names:
            raise ValueError(
                f"vector column {requested!r} not found in parquet schema {list(schema_names)!r}"
            )
        return requested
    matches = [name for name in VECTOR_COLUMN_CANDIDATES if name in schema_names]
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        raise ValueError(
            f"multiple plausible vector columns found {matches!r}; pass --vector-column explicitly"
        )
    fallback = [name for name in schema_names if name != DEFAULT_ID_COLUMN]
    if len(fallback) == 1:
        return fallback[0]
    raise ValueError(
        f"could not infer vector column from parquet schema {list(schema_names)!r}; "
        "pass --vector-column explicitly"
    )


def _load_sorted_ids(dataset, id_column: str, *, needed_rows: int) -> list[int]:
    id_table = dataset.to_table(columns=[id_column])
    ids = [int(value) for value in id_table[id_column].to_pylist()]
    if len(ids) < needed_rows:
        raise ValueError(
            f"parquet only has {len(ids)} rows, but {needed_rows} rows are required "
            "for the selected profile"
        )
    if len(set(ids)) != len(ids):
        raise ValueError("duplicate ids detected in parquet input; canonical selection is undefined")
    ids.sort()
    return ids


def _load_selected_rows(dataset, id_column: str, vector_column: str, selected_ids: set[int], *, dim: int):
    rows_by_id: dict[int, list[float]] = {}
    scanner = dataset.scanner(columns=[id_column, vector_column], batch_size=4096)
    for batch in scanner.to_batches():
        ids = batch.column(0).to_pylist()
        vectors = batch.column(1).to_pylist()
        for row_id_raw, vector_raw in zip(ids, vectors):
            row_id = int(row_id_raw)
            if row_id not in selected_ids:
                continue
            if row_id in rows_by_id:
                raise ValueError(f"duplicate selected id {row_id} encountered during parquet scan")
            rows_by_id[row_id] = _coerce_vector(
                vector_raw,
                dim=dim,
                row_id=row_id,
                vector_column=vector_column,
            )
            if len(rows_by_id) == len(selected_ids):
                return rows_by_id
    missing = sorted(selected_ids.difference(rows_by_id))
    raise ValueError(f"failed to recover {len(missing)} selected ids from parquet scan: {missing[:8]!r}")


def _write_tsv(path: Path, ids: Sequence[int], rows_by_id: dict[int, list[float]]) -> FileManifest:
    sha = hashlib.sha256()
    first_id: int | None = None
    last_id: int | None = None
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for row_id in ids:
            line = f"{row_id}\t{_canonical_json_array(rows_by_id[row_id])}\n"
            if first_id is None:
                first_id = row_id
            last_id = row_id
            handle.write(line)
            sha.update(line.encode("utf-8"))
    return FileManifest(
        file=path.name,
        rows=len(ids),
        sha256=sha.hexdigest(),
        first_id=first_id,
        last_id=last_id,
    )


def _write_manifest(
    path: Path,
    *,
    profile: SubsetProfile,
    source_parquet: str,
    source_dataset: str,
    id_column: str,
    vector_column: str,
    dim: int,
    corpus_manifest: FileManifest,
    query_manifest: FileManifest,
) -> None:
    manifest = {
        "manifest_version": 1,
        "prefix": profile.prefix,
        "source_dataset": source_dataset,
        "source_parquet": source_parquet,
        "id_column": id_column,
        "vector_column": vector_column,
        "dimension": dim,
        "selection_rule": {
            "sort_key": "id ascending",
            "corpus_start": 0,
            "corpus_rows": profile.corpus_rows,
            "query_start": profile.query_start,
            "query_rows": profile.query_rows,
        },
        "corpus": asdict(corpus_manifest),
        "queries": asdict(query_manifest),
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "generated_by": "scripts/qdrant_dbpedia_to_tsv.py",
    }
    with path.open("w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)
        handle.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Convert the Qdrant DBpedia parquet release into canonical TSV fixtures.",
    )
    parser.add_argument(
        "--profile",
        required=True,
        choices=sorted(PROFILES),
        help="Canonical subset profile to emit.",
    )
    parser.add_argument(
        "--parquet",
        required=True,
        help="Path to the Qdrant DBpedia parquet file or directory.",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write the canonical TSV and manifest files into.",
    )
    parser.add_argument(
        "--id-column",
        default=DEFAULT_ID_COLUMN,
        help=f"Parquet id column name (default: {DEFAULT_ID_COLUMN})",
    )
    parser.add_argument(
        "--vector-column",
        help=(
            "Parquet vector column name. If omitted, the script tries common "
            f"candidates {VECTOR_COLUMN_CANDIDATES!r}."
        ),
    )
    parser.add_argument(
        "--dim",
        type=int,
        default=DEFAULT_DIM,
        help=f"Expected embedding dimensionality (default: {DEFAULT_DIM})",
    )
    parser.add_argument(
        "--source-dataset",
        default=DEFAULT_SOURCE_DATASET,
        help=f"Dataset label stored in the manifest (default: {DEFAULT_SOURCE_DATASET!r})",
    )

    args = parser.parse_args()
    profile = PROFILES[args.profile]
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    ds = _import_pyarrow()
    dataset = ds.dataset(args.parquet, format="parquet")
    vector_column = _resolve_vector_column(dataset.schema.names, args.vector_column)
    needed_rows = profile.corpus_rows + profile.query_rows
    sorted_ids = _load_sorted_ids(dataset, args.id_column, needed_rows=needed_rows)

    corpus_ids = sorted_ids[: profile.corpus_rows]
    query_ids = sorted_ids[profile.query_start : profile.query_start + profile.query_rows]
    selected_rows = _load_selected_rows(
        dataset,
        args.id_column,
        vector_column,
        set(corpus_ids) | set(query_ids),
        dim=args.dim,
    )

    corpus_path = output_dir / f"{profile.prefix}_corpus.tsv"
    queries_path = output_dir / f"{profile.prefix}_queries.tsv"
    manifest_path = output_dir / f"{profile.prefix}_manifest.json"

    corpus_manifest = _write_tsv(corpus_path, corpus_ids, selected_rows)
    query_manifest = _write_tsv(queries_path, query_ids, selected_rows)
    _write_manifest(
        manifest_path,
        profile=profile,
        source_parquet=os.path.abspath(args.parquet),
        source_dataset=args.source_dataset,
        id_column=args.id_column,
        vector_column=vector_column,
        dim=args.dim,
        corpus_manifest=corpus_manifest,
        query_manifest=query_manifest,
    )

    print(f"[converter] wrote {corpus_path}", file=sys.stderr)
    print(f"[converter] wrote {queries_path}", file=sys.stderr)
    print(f"[converter] wrote {manifest_path}", file=sys.stderr)
    print(
        f"[converter] profile={profile.prefix} corpus_rows={profile.corpus_rows} "
        f"query_rows={profile.query_rows} sort_key='id ascending'",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
