#!/usr/bin/env python3
"""Load a real-corpus recall fixture into Postgres for tqhnsw A4 measurement.

This script implements the local-file loader path described in
``docs/RECALL_REAL_CORPUS.md``. It is the bridge between a staged
``<basename>_corpus.tsv`` / ``<basename>_queries.tsv`` pair on disk and the
SQL-side recall probes exposed by the extension.

It is intentionally idempotent so the recall lane can preserve the one-time
load / one-time index build / repeated rerun discipline used by the synthetic
fixture-backed gate.

Usage:

    PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \\
        --prefix tqhnsw_real_50k \\
        --corpus-file /path/to/dbpedia_50k_corpus.tsv \\
        --queries-file /path/to/dbpedia_1k_queries.tsv \\
        --m 8 16

The corpus and query files MUST follow the contract in
``docs/RECALL_REAL_CORPUS.md``: each line is ``<id>\\t<json_array>`` where
``<json_array>`` is a JSON array of floats, no header row.

The script does NOT download datasets. The repo never checks in dataset
binaries. The user is expected to stage the files out-of-band.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from typing import Iterable, List, Sequence

DEFAULT_BITS = 4
DEFAULT_SEED = 42
DEFAULT_EF_CONSTRUCTION = 128
SQL_IDENT_RE = re.compile(r"^[a-zA-Z_][a-zA-Z0-9_]*$")


def _validate_ident(name: str, label: str) -> str:
    if not SQL_IDENT_RE.match(name):
        raise ValueError(
            f"{label} {name!r} must match [a-zA-Z_][a-zA-Z0-9_]* (no quoting allowed)"
        )
    return name


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
    database: str, index: str, m: int, ef_construction: int
) -> bool:
    expected_m = f"m={m}"
    expected_ef = f"ef_construction={ef_construction}"
    expected_src = "build_source_column=source"
    sql = (
        "SELECT EXISTS (SELECT 1 FROM pg_class "
        f"WHERE relname = '{index}' AND relkind = 'i' "
        f"AND reloptions @> ARRAY['{expected_m}', '{expected_ef}', '{expected_src}'])"
    )
    return _psql(database, sql, capture=True).lower() == "t"


def _drop_relation_if_exists(database: str, name: str, kind: str) -> None:
    keyword = "INDEX" if kind == "i" else "TABLE"
    _psql(database, f"DROP {keyword} IF EXISTS {name} CASCADE")


def _read_vector_file(path: str, dim: int) -> Iterable[tuple[int, List[float]]]:
    with open(path, "r", encoding="utf-8") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.rstrip("\r\n")
            if not line:
                continue
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
                raise ValueError(
                    f"{path}:{line_number}: expected dim {dim}, got {len(values)}"
                )
            yield row_id, [float(v) for v in values]


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
) -> int:
    # COPY ... FROM STDIN as text. Each row: id\tsource_real_array
    # Then materialize embedding via UPDATE in one shot to avoid per-row SPI
    # overhead.
    print(f"[loader] inserting corpus rows into {table} ...", file=sys.stderr)
    _psql_copy(
        database,
        f"COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        (
            f"{row_id}\t{_format_real_array_literal(values)}"
            for row_id, values in _read_vector_file(path, dim)
        ),
    )
    inserted = _table_row_count(database, table)
    print(
        f"[loader] encoding tqvector embedding column for {inserted} rows in {table} ...",
        file=sys.stderr,
    )
    _psql(
        database,
        f"UPDATE {table} SET embedding = encode_to_tqvector(source, {bits}, {seed})",
    )
    return inserted


def _load_queries(database: str, table: str, path: str, dim: int) -> int:
    print(f"[loader] inserting query rows into {table} ...", file=sys.stderr)
    _psql_copy(
        database,
        f"COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        (
            f"{row_id}\t{_format_real_array_literal(values)}"
            for row_id, values in _read_vector_file(path, dim)
        ),
    )
    return _table_row_count(database, table)


def _ensure_corpus_table(
    database: str,
    table: str,
    path: str,
    dim: int,
    bits: int,
    seed: int,
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
            embedding tqvector
        )
        """,
    )
    return _load_corpus(database, table, path, dim, bits, seed)


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
    m: int,
    ef_construction: int,
) -> None:
    if _index_exists_with_options(database, index_name, m, ef_construction):
        print(
            f"[loader] {index_name} already exists with m={m} ef_construction={ef_construction}; skipping rebuild",
            file=sys.stderr,
        )
        return
    print(
        f"[loader] building {index_name} (m={m}, ef_construction={ef_construction}) ...",
        file=sys.stderr,
    )
    _psql(
        database,
        f"""
        CREATE INDEX {index_name} ON {corpus_table}
        USING tqhnsw (embedding tqvector_ip_ops)
        WITH (m = {m}, ef_construction = {ef_construction}, build_source_column = 'source')
        """,
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Load a real-corpus recall fixture into Postgres for tqhnsw A4 measurement.",
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
        "--bits",
        type=int,
        default=DEFAULT_BITS,
        help=f"Quantization bits passed to encode_to_tqvector (default {DEFAULT_BITS})",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_SEED,
        help=f"Quantizer seed passed to encode_to_tqvector (default {DEFAULT_SEED})",
    )
    parser.add_argument(
        "--ef-construction",
        type=int,
        default=DEFAULT_EF_CONSTRUCTION,
        help=f"ef_construction passed to CREATE INDEX (default {DEFAULT_EF_CONSTRUCTION})",
    )
    parser.add_argument(
        "--m",
        type=int,
        nargs="+",
        default=[8, 16],
        help="m values to build indexes for (default: 8 16)",
    )
    parser.add_argument(
        "--database",
        default=os.environ.get("PGDATABASE", "tqvector_bench"),
        help="PostgreSQL database name (defaults to $PGDATABASE or 'tqvector_bench')",
    )

    args = parser.parse_args()

    try:
        prefix = _validate_ident(args.prefix, "fixture prefix")
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    corpus_table = f"{prefix}_corpus"
    queries_table = f"{prefix}_queries"

    try:
        corpus_rows = _ensure_corpus_table(
            args.database,
            corpus_table,
            args.corpus_file,
            args.dim,
            args.bits,
            args.seed,
        )
        query_rows = _ensure_queries_table(
            args.database,
            queries_table,
            args.queries_file,
            args.dim,
        )
        for m_value in args.m:
            index_name = f"{prefix}_m{m_value}_idx"
            _ensure_index(
                args.database,
                corpus_table,
                index_name,
                m_value,
                args.ef_construction,
            )
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print(
        f"[loader] done. corpus={corpus_table} ({corpus_rows} rows), "
        f"queries={queries_table} ({query_rows} rows), m={args.m}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
