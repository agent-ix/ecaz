#!/usr/bin/env python3
"""Convert the Qdrant DBpedia parquet release into canonical TSV fixtures.

This script implements the reproducible subset-selection contract documented in
``docs/RECALL_REAL_CORPUS.md``:

- sort the full parquet release by the source parquet id column ascending
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
import heapq
import hashlib
import json
import math
import os
from pathlib import Path
import sys
from typing import Any, Sequence


DEFAULT_SOURCE_DATASET = "Qdrant dbpedia-entities-openai3-text-embedding-3-large-1536-1M"
DEFAULT_DIM = 1536
ID_COLUMN_CANDIDATES = ("id", "_id")
VECTOR_COLUMN_CANDIDATES = (
    "embedding",
    "vector",
    "values",
    "openai",
    "text_embedding",
    "text-embedding-3-large-1536-embedding",
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
    # One-time external oracle, see docs/RECALL_ANN_BENCHMARKS_ANCHOR.md.
    # The full Qdrant DBpedia 1M parquet ships exactly 1,000,000 rows with no
    # separate query split, so the anchor profile carves the last 10k rows out
    # of the canonical sorted-id ordering as the query set and leaves the
    # remaining 990k rows as the corpus.
    "tqhnsw_real_ann_benchmarks_anchor": SubsetProfile(
        prefix="tqhnsw_real_ann_benchmarks_anchor",
        corpus_rows=990_000,
        query_rows=10_000,
    ),
}


@dataclass
class FileManifest:
    file: str
    rows: int
    sha256: str
    first_id: int | None
    last_id: int | None
    first_source_id: str | None = None
    last_source_id: str | None = None


@dataclass(frozen=True)
class _ReverseSourceId:
    value: str

    def __lt__(self, other: "_ReverseSourceId") -> bool:
        return self.value > other.value


def _import_pyarrow():
    try:
        import pyarrow.parquet as pq
    except ModuleNotFoundError as exc:
        raise SystemExit(
            "pyarrow is required for parquet conversion. Install it locally before "
            "running scripts/qdrant_dbpedia_to_tsv.py."
        ) from exc
    return pq


def _resolve_parquet_files(parquet_path: str) -> list[Path]:
    path = Path(parquet_path)
    if path.is_file():
        return [path]
    if path.is_dir():
        files = sorted(path.glob("*.parquet"))
        if files:
            return files
    raise ValueError(f"no parquet files found at {parquet_path!r}")


def _iter_parquet_batches(parquet_files: Sequence[Path], columns: Sequence[str], *, batch_size: int):
    pq = _import_pyarrow()
    for parquet_file in parquet_files:
        parquet_reader = pq.ParquetFile(parquet_file)
        for batch in parquet_reader.iter_batches(columns=list(columns), batch_size=batch_size):
            yield batch


def _parquet_schema_names(parquet_files: Sequence[Path]) -> list[str]:
    pq = _import_pyarrow()
    if not parquet_files:
        raise ValueError("parquet file list must not be empty")
    return list(pq.ParquetFile(parquet_files[0]).schema_arrow.names)


def _canonical_float(value: float) -> str:
    if not math.isfinite(value):
        raise ValueError(f"non-finite value {value!r} is not allowed in embeddings")
    return format(float(value), ".9g")


def _canonical_json_array(values: Sequence[float]) -> str:
    return "[" + ",".join(_canonical_float(value) for value in values) + "]"


def _normalize_source_id(value: Any, *, id_column: str) -> str:
    if value is None:
        raise ValueError(f"id column {id_column!r} contains a null value")
    if isinstance(value, bytes):
        return value.decode("utf-8")
    return str(value)


def _coerce_vector(value, *, dim: int, row_id: str, vector_column: str) -> list[float]:
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
    fallback = [name for name in schema_names if name not in ID_COLUMN_CANDIDATES]
    if len(fallback) == 1:
        return fallback[0]
    raise ValueError(
        f"could not infer vector column from parquet schema {list(schema_names)!r}; "
        "pass --vector-column explicitly"
    )


def _resolve_id_column(schema_names: Sequence[str], requested: str | None) -> str:
    if requested:
        if requested not in schema_names:
            raise ValueError(
                f"id column {requested!r} not found in parquet schema {list(schema_names)!r}"
            )
        return requested
    matches = [name for name in ID_COLUMN_CANDIDATES if name in schema_names]
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        raise ValueError(
            f"multiple plausible id columns found {matches!r}; pass --id-column explicitly"
        )
    raise ValueError(
        f"could not infer id column from parquet schema {list(schema_names)!r}; "
        "pass --id-column explicitly"
    )


def _load_sorted_ids(parquet_files: Sequence[Path], id_column: str, *, needed_rows: int) -> list[str]:
    row_count = 0
    smallest_ids_heap: list[_ReverseSourceId] = []
    for batch in _iter_parquet_batches(parquet_files, [id_column], batch_size=16_384):
        for value in batch.column(0).to_pylist():
            row_count += 1
            source_id = _normalize_source_id(value, id_column=id_column)
            if len(smallest_ids_heap) < needed_rows:
                heapq.heappush(smallest_ids_heap, _ReverseSourceId(source_id))
                continue
            if source_id < smallest_ids_heap[0].value:
                heapq.heapreplace(smallest_ids_heap, _ReverseSourceId(source_id))
    if row_count < needed_rows:
        raise ValueError(
            f"parquet only has {row_count} rows, but {needed_rows} rows are required "
            "for the selected profile"
        )
    smallest_ids = sorted(item.value for item in smallest_ids_heap)
    if len(set(smallest_ids)) != len(smallest_ids):
        raise ValueError(
            "duplicate ids detected within the selected canonical prefix; selection is undefined"
        )
    return smallest_ids


def _load_selected_rows(
    parquet_files: Sequence[Path],
    id_column: str,
    vector_column: str,
    selected_ids: set[str],
    *,
    dim: int,
):
    rows_by_id: dict[str, list[float]] = {}
    for batch in _iter_parquet_batches(parquet_files, [id_column, vector_column], batch_size=4096):
        ids = batch.column(0).to_pylist()
        vectors = batch.column(1).to_pylist()
        for row_id_raw, vector_raw in zip(ids, vectors):
            row_id = _normalize_source_id(row_id_raw, id_column=id_column)
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


def _write_tsv(
    path: Path,
    entries: Sequence[tuple[int, str]],
    rows_by_id: dict[str, list[float]],
) -> FileManifest:
    sha = hashlib.sha256()
    first_id: int | None = None
    last_id: int | None = None
    first_source_id: str | None = None
    last_source_id: str | None = None
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for row_id, source_id in entries:
            line = f"{row_id}\t{_canonical_json_array(rows_by_id[source_id])}\n"
            if first_id is None:
                first_id = row_id
                first_source_id = source_id
            last_id = row_id
            last_source_id = source_id
            handle.write(line)
            sha.update(line.encode("utf-8"))
    return FileManifest(
        file=path.name,
        rows=len(entries),
        sha256=sha.hexdigest(),
        first_id=first_id,
        last_id=last_id,
        first_source_id=first_source_id,
        last_source_id=last_source_id,
    )


def _source_parquet_basename(source_parquet: str) -> str:
    # `Path(...).name` handles a trailing slash on directory inputs correctly
    # (`Path("/a/b/").name == "b"`), unlike `os.path.basename` which would
    # return an empty string for that form.
    return Path(source_parquet).name


def _source_parquet_shard_basenames(parquet_files: Sequence[Path]) -> list[str]:
    return sorted(parquet_file.name for parquet_file in parquet_files)


def _write_manifest(
    path: Path,
    *,
    profile: SubsetProfile,
    source_parquet: str,
    parquet_files: Sequence[Path],
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
        # `source_parquet` is the absolute path on the developer machine that
        # generated this manifest. It is kept as a local-debug hint only.
        # Reviewers on a different machine verify against the portable
        # `source_parquet_basename` / `source_parquet_shard_basenames` fields
        # below instead.
        "source_parquet": source_parquet,
        "source_parquet_basename": _source_parquet_basename(source_parquet),
        "source_parquet_shard_basenames": _source_parquet_shard_basenames(parquet_files),
        "id_column": id_column,
        "vector_column": vector_column,
        "dimension": dim,
        "selection_rule": {
            "sort_key": f"{id_column} ascending lexicographic",
            "corpus_start": 0,
            "corpus_rows": profile.corpus_rows,
            "query_start": profile.query_start,
            "query_rows": profile.query_rows,
            "output_id_mode": "global_sorted_row_index",
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
        help=(
            "Parquet id column name. If omitted, the script tries common "
            f"candidates {ID_COLUMN_CANDIDATES!r}."
        ),
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
        help=(
            "Human-readable dataset label stored in the manifest (not a path). "
            f"Default: {DEFAULT_SOURCE_DATASET!r}"
        ),
    )

    args = parser.parse_args()
    profile = PROFILES[args.profile]
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    parquet_files = _resolve_parquet_files(args.parquet)
    schema_names = _parquet_schema_names(parquet_files)
    id_column = _resolve_id_column(schema_names, args.id_column)
    vector_column = _resolve_vector_column(
        schema_names,
        args.vector_column,
    )
    needed_rows = profile.corpus_rows + profile.query_rows
    sorted_ids = _load_sorted_ids(parquet_files, id_column, needed_rows=needed_rows)

    corpus_source_ids = sorted_ids[: profile.corpus_rows]
    query_source_ids = sorted_ids[
        profile.query_start : profile.query_start + profile.query_rows
    ]
    selected_rows = _load_selected_rows(
        parquet_files,
        id_column,
        vector_column,
        set(corpus_source_ids) | set(query_source_ids),
        dim=args.dim,
    )
    corpus_entries = list(enumerate(corpus_source_ids, start=0))
    query_entries = list(enumerate(query_source_ids, start=profile.query_start))

    corpus_path = output_dir / f"{profile.prefix}_corpus.tsv"
    queries_path = output_dir / f"{profile.prefix}_queries.tsv"
    manifest_path = output_dir / f"{profile.prefix}_manifest.json"

    corpus_manifest = _write_tsv(corpus_path, corpus_entries, selected_rows)
    query_manifest = _write_tsv(queries_path, query_entries, selected_rows)
    _write_manifest(
        manifest_path,
        profile=profile,
        source_parquet=os.path.abspath(args.parquet),
        parquet_files=parquet_files,
        source_dataset=args.source_dataset,
        id_column=id_column,
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
        f"query_rows={profile.query_rows} sort_key='{id_column} ascending lexicographic'",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
