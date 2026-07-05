from pathlib import Path
import subprocess

CHUNKS = [
    ("contracts.rs", 1, 2865),
    ("tuple_heap.rs", 2866, 3587),
    ("coordinator_catalog.rs", 3588, 5150),
    ("production_summary.rs", 5151, 6004),
    ("transport_faults.rs", 6005, 6243),
    ("receive_faults.rs", 6244, 7181),
    ("libpq_executor.rs", 7182, 8448),
    ("node_catalog.rs", 8449, 9589),
    ("epoch_manifest.rs", 9590, 11342),
    ("catalog_cleanup_policy.rs", 11343, None),
]


def trim_trailing_blank_lines(text: str) -> str:
    lines = text.splitlines()
    while lines and lines[-1].strip() == "":
        lines.pop()
    return "\n".join(lines) + "\n"


old = subprocess.check_output(["git", "show", "HEAD~2:src/tests/remote_search.rs"]).decode()
old_lines = old.splitlines(keepends=True)

expected_parts = []
for _name, start, end in CHUNKS:
    part = "".join(old_lines[start - 1 : end])
    expected_parts.append(trim_trailing_blank_lines(part))
expected = "".join(expected_parts)

new_parts = []
for name, _start, _end in CHUNKS:
    text = (Path("src/tests/remote_search") / name).read_text()
    if name == "node_catalog.rs":
        text = text.replace(
            'include_str!("../../../sql/bootstrap.sql")',
            'include_str!("../../sql/bootstrap.sql")',
        )
        text = text.replace(
            'include_str!("../../../ecaz--0.1.0--0.1.1.sql")',
            'include_str!("../../ecaz--0.1.0--0.1.1.sql")',
        )
    new_parts.append(text)
new = "".join(new_parts)

print("normalized_content_match=", expected == new)
print("old_remote_search_lines=", len(old.splitlines()))
print("new_concatenated_lines=", len(new.splitlines()))
print("chunk_count=", len(CHUNKS))
