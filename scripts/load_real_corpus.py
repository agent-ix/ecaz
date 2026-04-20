#!/usr/bin/env python3
"""Load a real-corpus fixture into Postgres for Ecaz access-method evaluation.

This script implements the local-file loader path described in
``docs/RECALL_REAL_CORPUS.md``. It is the bridge between a staged
``<basename>_corpus.tsv`` / ``<basename>_queries.tsv`` pair on disk and the
Postgres-side corpus/query tables used by the extension's recall and benchmark
surfaces.

It is intentionally idempotent so the real-corpus lane can preserve the
one-time load / one-time index build / repeated rerun discipline used by the
synthetic fixture-backed gate.

Usage:

    PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \\
        --prefix ec_hnsw_real_50k \\
        --corpus-file /path/to/dbpedia_50k_corpus.tsv \\
        --queries-file /path/to/dbpedia_1k_queries.tsv \\
        --m 8 16

    PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \\
        --prefix ec_diskann_real_10k \\
        --corpus-file /path/to/dbpedia_10k_corpus.tsv \\
        --queries-file /path/to/dbpedia_200_queries.tsv \\
        --index-profile ec_diskann

The corpus and query files MUST follow the contract in
``docs/RECALL_REAL_CORPUS.md``: each line is ``<id>\\t<json_array>`` where
``<json_array>`` is a JSON array of floats, no header row.

The script does NOT download datasets. The repo never checks in dataset
binaries. The user is expected to stage the files out-of-band.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import sys
from typing import Iterable, List, Sequence

DEFAULT_BITS = 4
DEFAULT_SEED = 42
DEFAULT_EF_CONSTRUCTION = 128
DEFAULT_BUILD_SOURCE_COLUMN = "source"
DEFAULT_EMBEDDING_TYPE = "ecvector"
DEFAULT_ENCODER_FUNCTION = "encode_to_ecvector"
UNIT_NORM_TOLERANCE = 0.05
SQL_IDENT_RE = re.compile(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
VALID_STORAGE_FORMATS = frozenset(("turboquant", "pq_fastscan"))
SQL_NUMERIC_RE = re.compile(r"^[+-]?(?:\d+(?:\.\d+)?|\.\d+)$")
SQL_BOOLEAN_LITERALS = frozenset(("true", "false"))


@dataclass(frozen=True)
class IndexProfile:
    name: str
    access_method: str
    operator_class: str
    embedding_type: str
    encoder_function: str
    supports_build_source_column: bool
    supports_m_sweep: bool


INDEX_PROFILES = {
    "ec_hnsw": IndexProfile(
        name="ec_hnsw",
        access_method="ec_hnsw",
        operator_class="ecvector_ip_ops",
        embedding_type="ecvector",
        encoder_function="encode_to_ecvector",
        supports_build_source_column=True,
        supports_m_sweep=True,
    ),
    "ec_diskann": IndexProfile(
        name="ec_diskann",
        access_method="ec_diskann",
        operator_class="ecvector_diskann_ip_ops",
        embedding_type="ecvector",
        encoder_function="encode_to_ecvector",
        supports_build_source_column=False,
        supports_m_sweep=False,
    ),
}
DEFAULT_INDEX_PROFILE = "ec_hnsw"


@dataclass
class VectorNormStats:
    label: str
    count: int = 0
    mean_norm: float = 0.0
    min_norm: float = float("inf")
    max_norm: float = 0.0

    def observe(self, values: Sequence[float]) -> None:
        norm = math.sqrt(sum(float(v) * float(v) for v in values))
        self.count += 1
        self.mean_norm += (norm - self.mean_norm) / self.count
        self.min_norm = min(self.min_norm, norm)
        self.max_norm = max(self.max_norm, norm)

    def log(self) -> None:
        if self.count == 0:
            return
        print(
            f"[loader] {self.label} norms: count={self.count} "
            f"mean={self.mean_norm:.6f} min={self.min_norm:.6f} max={self.max_norm:.6f}",
            file=sys.stderr,
        )
        if (
            abs(self.mean_norm - 1.0) > UNIT_NORM_TOLERANCE
            or self.min_norm < 1.0 - UNIT_NORM_TOLERANCE
            or self.max_norm > 1.0 + UNIT_NORM_TOLERANCE
        ):
            print(
                f"[loader] warning: {self.label} vectors do not appear unit-normalized; "
                "inner-product/cosine benchmark assumptions may not hold",
                file=sys.stderr,
            )


@dataclass
class VectorFileStats:
    rows: int
    sha256: str
    first_id: int | None
    last_id: int | None


def _validate_ident(name: str, label: str) -> str:
    if not SQL_IDENT_RE.match(name):
        raise ValueError(
            f"{label} {name!r} must match [a-zA-Z_][a-zA-Z0-9_]* (no quoting allowed)"
        )
    return name


def _validate_storage_format(raw_value: str) -> str:
    if raw_value not in VALID_STORAGE_FORMATS:
        allowed = ", ".join(sorted(VALID_STORAGE_FORMATS))
        raise argparse.ArgumentTypeError(
            f"storage format {raw_value!r} must be one of: {allowed}"
        )
    return raw_value


def _validate_index_profile(raw_value: str) -> str:
    if raw_value not in INDEX_PROFILES:
        allowed = ", ".join(sorted(INDEX_PROFILES))
        raise argparse.ArgumentTypeError(
            f"index profile {raw_value!r} must be one of: {allowed}"
        )
    return raw_value


def _validate_reloption(raw_value: str) -> str:
    key, sep, value = raw_value.partition("=")
    if sep != "=" or not key or not value:
        raise argparse.ArgumentTypeError(
            f"index reloption {raw_value!r} must look like key=value"
        )
    _validate_ident(key, "index reloption key")
    return f"{key}={value}"


def _resolve_index_profile(name: str) -> IndexProfile:
    return INDEX_PROFILES[name]


def _index_prefix(prefix: str, storage_format: str | None) -> str:
    if storage_format is None:
        return prefix
    return f"{prefix}_{storage_format}"


def _index_name(index_prefix: str, m: int) -> str:
    return f"{index_prefix}_m{m}_idx"


def _format_hnsw_reloptions(
    m: int,
    ef_construction: int,
    storage_format: str | None,
    build_source_column: str,
) -> list[str]:
    reloptions = [
        f"m={m}",
        f"ef_construction={ef_construction}",
        f"build_source_column={build_source_column}",
    ]
    if storage_format is not None:
        reloptions.append(f"storage_format={storage_format}")
    return reloptions


def _parse_reloption(reloption: str) -> tuple[str, str]:
    key, sep, value = reloption.partition("=")
    if sep != "=" or not key or not value:
        raise ValueError(f"invalid reloption {reloption!r}: expected key=value")
    return key, value


def _format_reloption_sql_value(raw_value: str) -> str:
    if raw_value.startswith("'") and raw_value.endswith("'"):
        return raw_value
    lowered = raw_value.lower()
    if lowered in SQL_BOOLEAN_LITERALS or SQL_NUMERIC_RE.match(raw_value):
        return raw_value
    return "'" + raw_value.replace("'", "''") + "'"


def _expected_index_reloptions(reloptions: Sequence[str]) -> list[str]:
    return [f"{key}={value}" for key, value in (_parse_reloption(item) for item in reloptions)]


def _build_index_sql(
    corpus_table: str,
    index_name: str,
    access_method: str,
    operator_class: str,
    reloptions: Sequence[str],
) -> str:
    joined_options = ", ".join(
        f"{key} = {_format_reloption_sql_value(value)}"
        for key, value in (_parse_reloption(item) for item in reloptions)
    )
    with_clause = f"\n        WITH ({joined_options})" if joined_options else ""
    return (
        f"CREATE INDEX {index_name} ON {corpus_table}\n"
        f"        USING {access_method} (embedding {operator_class})"
        f"{with_clause}"
    )


def _resolve_psql_bin() -> str:
    env_path = os.environ.get("TQV_PSQL_BIN")
    if env_path:
        return env_path
    psql = shutil.which("psql")
    if psql:
        return psql
    raise FileNotFoundError(
        "psql not found on PATH; set TQV_PSQL_BIN to an explicit psql binary"
    )


def _psql(database: str, sql: str, *, capture: bool = False) -> str:
    cmd = [_resolve_psql_bin(), database, "-v", "ON_ERROR_STOP=1"]
    if capture:
        cmd.extend(["-t", "-A", "-c", sql])
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout.strip()
    cmd.extend(["-q", "-c", sql])
    subprocess.run(cmd, check=True)
    return ""


def _psql_copy(database: str, sql: str, payload: Iterable[str]) -> None:
    cmd = [_resolve_psql_bin(), database, "-v", "ON_ERROR_STOP=1", "-q", "-c", sql]
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, text=True)
    assert proc.stdin is not None
    try:
        for line in payload:
            proc.stdin.write(line)
            if not line.endswith("\n"):
                proc.stdin.write("\n")
    finally:
        proc.stdin.close()
    rc = proc.wait()
    if rc != 0:
        raise RuntimeError(f"psql COPY exited with status {rc}")


def _table_exists(database: str, table: str) -> bool:
    out = _psql(
        database,
        f"SELECT EXISTS (SELECT 1 FROM pg_class WHERE relname = '{table}' AND relkind = 'r')",
        capture=True,
    )
    return out.lower() == "t"


def _table_row_count(database: str, table: str) -> int:
    out = _psql(database, f"SELECT count(*) FROM {table}", capture=True)
    return int(out)


def _index_exists_with_options(
    database: str,
    index: str,
    reloptions: Sequence[str],
) -> bool:
    expected = _expected_index_reloptions(reloptions)
    if not expected:
        sql = (
            "SELECT EXISTS (SELECT 1 FROM pg_class "
            f"WHERE relname = '{index}' AND relkind = 'i')"
        )
        return _psql(database, sql, capture=True).lower() == "t"
    expected_joined = "', '".join(expected)
    sql = (
        "SELECT EXISTS (SELECT 1 FROM pg_class "
        f"WHERE relname = '{index}' AND relkind = 'i' "
        f"AND reloptions @> ARRAY['{expected_joined}'])"
    )
    return _psql(database, sql, capture=True).lower() == "t"


def _drop_relation_if_exists(database: str, name: str, kind: str) -> None:
    keyword = "INDEX" if kind == "i" else "TABLE"
    _psql(database, f"DROP {keyword} IF EXISTS {name} CASCADE")


def _parse_vector_line(path: str, line_number: int, line: str, dim: int) -> tuple[int, List[float]]:
    try:
        id_str, json_str = line.split("\t", 1)
    except ValueError as exc:
        raise ValueError(
            f"{path}:{line_number}: expected '<id>\\t<json_array>' line, got {line!r}"
        ) from exc
    try:
        row_id = int(id_str)
    except ValueError as exc:
        raise ValueError(f"{path}:{line_number}: id {id_str!r} is not an integer") from exc
    try:
        values = json.loads(json_str)
    except json.JSONDecodeError as exc:
        raise ValueError(
            f"{path}:{line_number}: embedding column is not valid JSON ({exc})"
        ) from exc
    if not isinstance(values, list):
        raise ValueError(
            f"{path}:{line_number}: embedding must be a JSON array, got {type(values).__name__}"
        )
    if len(values) != dim:
        raise ValueError(f"{path}:{line_number}: expected dim {dim}, got {len(values)}")
    return row_id, [float(v) for v in values]


def _read_vector_file(path: str, dim: int) -> Iterable[tuple[int, List[float]]]:
    with open(path, "r", encoding="utf-8") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.rstrip("\r\n")
            if not line:
                continue
            yield _parse_vector_line(path, line_number, line, dim)


def _inspect_vector_file(path: str, dim: int) -> VectorFileStats:
    sha = hashlib.sha256()
    rows = 0
    first_id: int | None = None
    last_id: int | None = None
    with open(path, "rb") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            sha.update(raw_line)
            line = raw_line.decode("utf-8").rstrip("\r\n")
            if not line:
                continue
            row_id, _ = _parse_vector_line(path, line_number, line, dim)
            if first_id is None:
                first_id = row_id
            last_id = row_id
            rows += 1
    return VectorFileStats(rows=rows, sha256=sha.hexdigest(), first_id=first_id, last_id=last_id)


def _derive_manifest_path(corpus_file: str, queries_file: str) -> str | None:
    corpus_suffix = "_corpus.tsv"
    queries_suffix = "_queries.tsv"
    if not corpus_file.endswith(corpus_suffix) or not queries_file.endswith(queries_suffix):
        return None
    corpus_base = corpus_file[: -len(corpus_suffix)]
    queries_base = queries_file[: -len(queries_suffix)]
    if corpus_base != queries_base:
        return None
    return corpus_base + "_manifest.json"


def _verify_manifest(
    manifest_path: str,
    prefix: str,
    corpus_file: str,
    queries_file: str,
    dim: int,
    *,
    allow_mismatch: bool,
) -> None:
    with open(manifest_path, "r", encoding="utf-8") as handle:
        manifest = json.load(handle)

    problems: list[str] = []
    if manifest.get("manifest_version") != 1:
        problems.append(
            f"manifest_version={manifest.get('manifest_version')!r} (expected 1)"
        )
    if manifest.get("prefix") != prefix:
        problems.append(f"prefix={manifest.get('prefix')!r} (expected {prefix!r})")
    if manifest.get("dimension") != dim:
        problems.append(f"dimension={manifest.get('dimension')!r} (expected {dim})")

    # Portable dataset-identity fields (additive, may be absent on older
    # manifests). The absolute `source_parquet` field is deliberately ignored
    # here: it is a local-debug hint only and is expected to reference a path
    # that does not exist on the reviewer's machine.
    if "source_parquet_basename" in manifest:
        basename_value = manifest.get("source_parquet_basename")
        if not isinstance(basename_value, str):
            problems.append(
                f"source_parquet_basename={basename_value!r} (expected string)"
            )
        elif "/" in basename_value or "\\" in basename_value:
            problems.append(
                f"source_parquet_basename={basename_value!r} "
                "(expected portable basename, not a path)"
            )
    if "source_parquet_shard_basenames" in manifest:
        shard_value = manifest.get("source_parquet_shard_basenames")
        if not isinstance(shard_value, list) or not all(
            isinstance(shard, str) for shard in shard_value
        ):
            problems.append(
                f"source_parquet_shard_basenames={shard_value!r} (expected list of strings)"
            )
        else:
            for shard in shard_value:
                if "/" in shard or "\\" in shard:
                    problems.append(
                        f"source_parquet_shard_basenames entry {shard!r} "
                        "(expected portable basename, not a path)"
                    )
                    break

    corpus_stats = _inspect_vector_file(corpus_file, dim)
    query_stats = _inspect_vector_file(queries_file, dim)
    checks = [
        ("corpus", corpus_file, corpus_stats),
        ("queries", queries_file, query_stats),
    ]
    for label, path, stats in checks:
        section = manifest.get(label, {})
        expected_basename = section.get("file")
        if expected_basename and expected_basename != os.path.basename(path):
            problems.append(
                f"{label}.file={expected_basename!r} "
                f"(expected {os.path.basename(path)!r})"
            )
        if section.get("rows") != stats.rows:
            problems.append(f"{label}.rows={section.get('rows')!r} (expected {stats.rows})")
        if section.get("sha256") != stats.sha256:
            problems.append(
                f"{label}.sha256={section.get('sha256')!r} (expected {stats.sha256})"
            )
        if section.get("first_id") != stats.first_id:
            problems.append(
                f"{label}.first_id={section.get('first_id')!r} (expected {stats.first_id})"
            )
        if section.get("last_id") != stats.last_id:
            problems.append(
                f"{label}.last_id={section.get('last_id')!r} (expected {stats.last_id})"
            )

    if problems:
        message = (
            f"manifest verification failed for {manifest_path}: " + "; ".join(problems)
        )
        if allow_mismatch:
            print(f"[loader] warning: {message}", file=sys.stderr)
            return
        raise ValueError(message)

    print(
        f"[loader] verified manifest {manifest_path} for prefix {prefix}",
        file=sys.stderr,
    )


def _format_real_array_literal(values: Sequence[float]) -> str:
    # Use the curly-brace COPY array literal: real[] inputs accept {a,b,c}
    # without the explicit ARRAY[...]::real[] cast. This keeps the COPY stream
    # parseable in a single allocation per row.
    return "{" + ",".join(repr(float(v)) for v in values) + "}"


def _load_corpus(
    database: str,
    table: str,
    path: str,
    dim: int,
    bits: int,
    seed: int,
    profile: IndexProfile,
) -> int:
    # COPY ... FROM STDIN as text. Each row: id\tsource_real_array
    # Then materialize embedding via UPDATE in one shot to avoid per-row SPI
    # overhead.
    norm_stats = VectorNormStats(label=f"{table} corpus")

    def corpus_payload() -> Iterable[str]:
        for row_id, values in _read_vector_file(path, dim):
            norm_stats.observe(values)
            yield f"{row_id}\t{_format_real_array_literal(values)}"

    print(f"[loader] inserting corpus rows into {table} ...", file=sys.stderr)
    _psql_copy(
        database,
        f"COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        corpus_payload(),
    )
    inserted = _table_row_count(database, table)
    norm_stats.log()
    print(
        f"[loader] encoding {profile.embedding_type} embedding column for {inserted} "
        f"rows in {table} via {profile.encoder_function}() ...",
        file=sys.stderr,
    )
    _psql(
        database,
        f"UPDATE {table} SET embedding = {profile.encoder_function}(source, {bits}, {seed})",
    )
    return inserted


def _load_queries(database: str, table: str, path: str, dim: int) -> int:
    norm_stats = VectorNormStats(label=f"{table} queries")

    def query_payload() -> Iterable[str]:
        for row_id, values in _read_vector_file(path, dim):
            norm_stats.observe(values)
            yield f"{row_id}\t{_format_real_array_literal(values)}"

    print(f"[loader] inserting query rows into {table} ...", file=sys.stderr)
    _psql_copy(
        database,
        f"COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        query_payload(),
    )
    norm_stats.log()
    return _table_row_count(database, table)


def _ensure_corpus_table(
    database: str,
    table: str,
    path: str,
    dim: int,
    bits: int,
    seed: int,
    profile: IndexProfile,
) -> int:
    if _table_exists(database, table):
        existing = _table_row_count(database, table)
        if existing > 0:
            print(
                f"[loader] {table} already has {existing} rows; skipping reload",
                file=sys.stderr,
            )
            return existing
        # Empty table from a half-finished previous run — drop and reload.
        print(f"[loader] {table} exists but is empty; dropping and reloading", file=sys.stderr)
        _drop_relation_if_exists(database, table, "r")
    _psql(
        database,
        f"""
        CREATE TABLE {table} (
            id        bigint PRIMARY KEY,
            source    real[] NOT NULL,
            embedding {profile.embedding_type}
        )
        """,
    )
    return _load_corpus(database, table, path, dim, bits, seed, profile)


def _ensure_queries_table(database: str, table: str, path: str, dim: int) -> int:
    if _table_exists(database, table):
        existing = _table_row_count(database, table)
        if existing > 0:
            print(
                f"[loader] {table} already has {existing} rows; skipping reload",
                file=sys.stderr,
            )
            return existing
        print(f"[loader] {table} exists but is empty; dropping and reloading", file=sys.stderr)
        _drop_relation_if_exists(database, table, "r")
    _psql(
        database,
        f"""
        CREATE TABLE {table} (
            id     bigint PRIMARY KEY,
            source real[] NOT NULL
        )
        """,
    )
    return _load_queries(database, table, path, dim)


def _ensure_index(
    database: str,
    corpus_table: str,
    index_name: str,
    profile: IndexProfile,
    reloptions: Sequence[str],
) -> None:
    reloption_summary = ", ".join(reloptions) if reloptions else "<none>"
    if _index_exists_with_options(database, index_name, reloptions):
        print(
            f"[loader] {index_name} already exists with reloptions=[{reloption_summary}]; "
            "skipping rebuild",
            file=sys.stderr,
        )
        return
    print(
        f"[loader] building {index_name} using {profile.access_method} "
        f"(reloptions=[{reloption_summary}]) ...",
        file=sys.stderr,
    )
    _psql(
        database,
        _build_index_sql(
            corpus_table,
            index_name,
            profile.access_method,
            profile.operator_class,
            reloptions,
        ),
    )


def _dedupe_int_sweep(groups: Sequence[Sequence[int]]) -> list[int]:
    result: list[int] = []
    seen: set[int] = set()
    for group in groups:
        for value in group:
            if value in seen:
                continue
            seen.add(value)
            result.append(value)
    return result


def _build_hnsw_reloption_sweep(
    m_values: Sequence[int],
    ef_construction: int,
    storage_format: str | None,
    build_source_column: str,
    extra_reloptions: Sequence[str],
) -> list[tuple[int, list[str]]]:
    sweep: list[tuple[int, list[str]]] = []
    for m_value in m_values:
        reloptions = _format_hnsw_reloptions(
            m_value,
            ef_construction,
            storage_format,
            build_source_column,
        )
        reloptions.extend(extra_reloptions)
        sweep.append((m_value, reloptions))
    return sweep


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Load a real-corpus fixture into Postgres for Ecaz access-method evaluation "
            "(ec_hnsw today, ec_diskann in progress, extensible to future AMs)."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--prefix",
        required=True,
        help="Fixture prefix used for table and index names. Must match [a-zA-Z_][a-zA-Z0-9_]*.",
    )
    parser.add_argument("--corpus-file", required=True, help="Path to <basename>_corpus.tsv")
    parser.add_argument("--queries-file", required=True, help="Path to <basename>_queries.tsv")
    parser.add_argument("--dim", type=int, default=1536, help="Vector dimensionality (default 1536)")
    parser.add_argument(
        "--index-profile",
        type=_validate_index_profile,
        default=DEFAULT_INDEX_PROFILE,
        help=(
            "Access-method profile to use. Controls embedding column type, encoder, "
            "operator class, and USING clause. "
            f"Choices: {', '.join(sorted(INDEX_PROFILES))} (default: {DEFAULT_INDEX_PROFILE})."
        ),
    )
    parser.add_argument(
        "--bits",
        type=int,
        default=DEFAULT_BITS,
        help=f"Quantization bits passed to the profile's encoder (default {DEFAULT_BITS})",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_SEED,
        help=f"Quantizer seed passed to the profile's encoder (default {DEFAULT_SEED})",
    )
    parser.add_argument(
        "--ef-construction",
        type=int,
        default=DEFAULT_EF_CONSTRUCTION,
        help=(
            f"ef_construction passed to CREATE INDEX for HNSW profiles "
            f"(default {DEFAULT_EF_CONSTRUCTION}). Ignored for profiles without an "
            "m-sweep (e.g. ec_diskann)."
        ),
    )
    parser.add_argument(
        "--m",
        type=int,
        nargs="+",
        action="append",
        default=None,
        help=(
            "m values to build indexes for. Accepts either '--m 8 16' or "
            "repeated '--m 8 --m 16' forms. HNSW profiles only (default: 8 16 for HNSW)."
        ),
    )
    parser.add_argument(
        "--reloption",
        action="append",
        default=[],
        type=_validate_reloption,
        metavar="key=value",
        help=(
            "Pass-through CREATE INDEX WITH (...) reloption. Repeatable. "
            "Use this for AM-specific tunables not covered by dedicated flags "
            "(e.g. --reloption graph_degree=48 --reloption alpha=1.2)."
        ),
    )
    parser.add_argument(
        "--database",
        default=os.environ.get("PGDATABASE", "tqvector_bench"),
        help="PostgreSQL database name (defaults to $PGDATABASE or 'tqvector_bench')",
    )
    parser.add_argument(
        "--manifest-file",
        help=(
            "Optional path to <basename>_manifest.json. If omitted, the loader "
            "auto-discovers a sibling manifest when the corpus/query files follow "
            "the canonical <basename>_{corpus,queries}.tsv naming pattern."
        ),
    )
    parser.add_argument(
        "--allow-manifest-mismatch",
        action="store_true",
        help="Continue after manifest verification fails, logging a warning instead of aborting.",
    )
    parser.add_argument(
        "--storage-format",
        type=_validate_storage_format,
        help=(
            "Optional storage format reloption. For ec_hnsw, the loader builds "
            "coexisting format-specific indexes named <prefix>_<storage_format>_m{N}_idx. "
            "For ec_diskann, only 'pq_fastscan' is accepted by the AM."
        ),
    )

    args = parser.parse_args()

    try:
        prefix = _validate_ident(args.prefix, "fixture prefix")
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    profile = _resolve_index_profile(args.index_profile)
    extra_reloptions: list[str] = list(args.reloption or [])

    if not profile.supports_m_sweep and args.m:
        print(
            f"error: --m is not supported by index profile {profile.name!r}; "
            "use --reloption key=value to pass AM-specific tunables",
            file=sys.stderr,
        )
        return 2

    corpus_table = f"{prefix}_corpus"
    queries_table = f"{prefix}_queries"
    storage_format = args.storage_format
    index_prefix = _index_prefix(prefix, storage_format)
    manifest_path = args.manifest_file or _derive_manifest_path(
        args.corpus_file, args.queries_file
    )

    if profile.supports_m_sweep:
        m_values = _dedupe_int_sweep(args.m) if args.m else [8, 16]
        index_plan = _build_hnsw_reloption_sweep(
            m_values,
            args.ef_construction,
            storage_format,
            DEFAULT_BUILD_SOURCE_COLUMN,
            extra_reloptions,
        )
        index_jobs = [
            (_index_name(index_prefix, m_value), reloptions)
            for m_value, reloptions in index_plan
        ]
        sweep_summary = f"m={m_values}"
    else:
        reloptions = list(extra_reloptions)
        if storage_format is not None:
            reloptions.append(f"storage_format={storage_format}")
        index_jobs = [(f"{index_prefix}_idx", reloptions)]
        sweep_summary = "single index"

    try:
        if manifest_path and os.path.exists(manifest_path):
            _verify_manifest(
                manifest_path,
                prefix,
                args.corpus_file,
                args.queries_file,
                args.dim,
                allow_mismatch=args.allow_manifest_mismatch,
            )
        elif args.manifest_file:
            raise FileNotFoundError(f"manifest file {args.manifest_file!r} does not exist")
        elif manifest_path:
            print(
                f"[loader] no sibling manifest found at {manifest_path}; continuing without manifest verification",
                file=sys.stderr,
            )
        corpus_rows = _ensure_corpus_table(
            args.database,
            corpus_table,
            args.corpus_file,
            args.dim,
            args.bits,
            args.seed,
            profile,
        )
        query_rows = _ensure_queries_table(
            args.database,
            queries_table,
            args.queries_file,
            args.dim,
        )
        for index_name, reloptions in index_jobs:
            _ensure_index(
                args.database,
                corpus_table,
                index_name,
                profile,
                reloptions,
            )
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print(
        f"[loader] done. profile={profile.name}, corpus={corpus_table} ({corpus_rows} rows), "
        f"queries={queries_table} ({query_rows} rows), index_prefix={index_prefix}, "
        f"storage_format={storage_format or 'default'}, {sweep_summary}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
