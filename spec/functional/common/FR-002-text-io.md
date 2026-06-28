---
id: FR-002
title: tqvector Text I/O
type: FR
status: APPROVED
object: api_endpoint
traces:
  - US-001
  - FR-001
---
# FR-002: tqvector Text I/O

## Description

The extension SHALL provide text input/output functions for the `tqvector` type.

### Text Format

```
[dim=<D>,bits=<B>,seed=<S>,gamma=<G>]:<hex_codes>
```

Example (4-dim, 4-bit, seed 42):
``` 
[dim=4,bits=4,seed=42,gamma=0.0]:000000
```

`<hex_codes>` encodes only the persisted `code_bytes` section (`mse_packed + qjl_packed`). The `gamma` scalar is represented by the named `gamma=<G>` field and is not included in the hex string.

For this requirement:
- `code_len(dim, bits) = ceil(dim * (bits-1) / 8) + ceil(dim / 8)`
- text I/O hex length in characters = `2 * code_len(dim, bits)`

### Input Function: `tqvector_in`

- SHALL parse the text format and produce the internal binary representation
- SHALL reject input where hex code length does not match `code_len(dim, bits)`
- SHALL reject input with missing `dim` or `bits` fields
- SHALL default `seed` to 42 if omitted
- SHALL default `gamma` to `0.0` if omitted
- SHALL produce a clear ERROR on invalid input (malformed brackets, bad hex, etc.)

### Output Function: `tqvector_out`

- SHALL produce the canonical text format from the internal binary representation
- SHALL always include `dim`, `bits`, `seed`, and `gamma` fields
- SHALL hex-encode the code bytes in lowercase

## Endpoint

The text I/O surface is registered as two `LANGUAGE c` SQL functions in `sql/bootstrap.sql`, backed by the parse/format helpers in `src/lib.rs`. Both are `IMMUTABLE STRICT PARALLEL SAFE`.

| Function | Signature | Direction | Description |
|---|---|---|---|
| `tqvector_in` | `tqvector_in(cstring) → tqvector` | cstring → tqvector | Parses the canonical text form `[dim=<D>,bits=<B>,seed=<S>,gamma=<G>]:<hex_codes>` and packs the internal datum. Defaults `seed` to 42 and `gamma` to 0.0 when omitted; errors on bad hex or a code length mismatch. |
| `tqvector_out` | `tqvector_out(tqvector) → cstring` | tqvector → cstring | Emits the canonical text form, always including `dim`, `bits`, `seed`, and `gamma`, with the code bytes hex-encoded in lowercase. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | `tqvector_out(tqvector_in(text))` produces the canonical form of any valid input text | Test |
| FR-002-AC-2 | Invalid hex input raises ERROR with a message containing "hex" | Test |
| FR-002-AC-3 | Hex length not matching `code_len(dim, bits)` raises ERROR with "code length mismatch" | Test |

### FR-002-AC-1: Text round-trip
`tqvector_out(tqvector_in(text))` SHALL produce the canonical form of any valid input text.

### FR-002-AC-2: Error on invalid hex
Input `'[dim=4,bits=4]:ZZZZ'::tqvector` SHALL raise ERROR with a message containing "hex".

### FR-002-AC-3: Error on dimension mismatch
Input with hex length not matching `code_len(dim, bits)` SHALL raise ERROR with "code length mismatch".

## Dependencies

- **Upstream**: US-001, FR-001 (traces)
- **Downstream**: none identified
