// Killing tests for mutation survivors in
// `src/am/ec_spire/storage/helpers.rs` surfaced by the packet 049
// manual cargo-mutants verification campaign. Each test targets one or
// more specific operator-swap mutations not killed by the cumulative
// careful suite at the start of 049.

#[test]
fn miri_validate_vec_id_bytes_accepts_max_length_global() {
    // Targets helpers.rs:5:20 (`>` -> `==`, `>` -> `>=`).
    // A 32-byte global vec_id (1 discriminator + 31 payload) is exactly
    // SPIRE_VEC_ID_MAX_BYTES. Original `bytes.len() > MAX` is false →
    // accept. Mutants `==` / `>=` are true at the boundary → reject.
    let mut bytes = vec![super::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR];
    bytes.extend_from_slice(&[0xab_u8; super::SPIRE_VEC_ID_MAX_BYTES - 1]);
    assert_eq!(bytes.len(), super::SPIRE_VEC_ID_MAX_BYTES);
    let vec_id = SpireVecId::from_bytes(&bytes)
        .expect("32-byte global vec_id must be accepted at the boundary");
    assert_eq!(vec_id.as_bytes().len(), super::SPIRE_VEC_ID_MAX_BYTES);
}

#[test]
fn miri_is_visible_primary_assignment_flags_rejects_zero_flags() {
    // Targets helpers.rs:43:11 (`&` -> `|`).
    // With flags=0, original `0 & PRIMARY != 0` = false → not visible.
    // Mutant `0 | PRIMARY != 0` = true → reports visible.
    // (The matching `|` -> `^` mutations at lines 40/41/42 are
    // equivalent — disjoint single-bit flags satisfy `a|b == a^b`.)
    assert!(!super::is_visible_primary_assignment_flags(0));
    // Sanity: PRIMARY alone IS visible.
    assert!(super::is_visible_primary_assignment_flags(
        SPIRE_ASSIGNMENT_FLAG_PRIMARY
    ));
}

#[test]
fn miri_is_visible_scored_assignment_flags_rejects_zero_flags() {
    // Targets helpers.rs:51:11 (`&` -> `|`).
    // Same shape as the primary-flags test above.
    assert!(!super::is_visible_scored_assignment_flags(0));
    // Sanity: PRIMARY alone IS scored-visible.
    assert!(super::is_visible_scored_assignment_flags(
        SPIRE_ASSIGNMENT_FLAG_PRIMARY
    ));
}

#[test]
fn miri_is_visible_scored_assignment_rejects_tombstone() {
    // Targets helpers.rs:59:5 (`is_visible_scored_assignment -> true`).
    // A row with TOMBSTONE set must NOT be visible-scored; the
    // always-true mutant would return true.
    let mut row = leaf_v2_assignment(1, 8);
    row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
    assert!(!is_visible_scored_assignment(&row));
}

#[test]
fn miri_is_visible_primary_assignment_ref_rejects_tombstone() {
    // Targets helpers.rs:65:5 (`is_visible_primary_assignment_ref -> true`).
    // Same shape as the scored test above but on the ref variant via
    // SpireLeafAssignmentRow::decode_prefix_ref.
    let mut row = leaf_v2_assignment(2, 8);
    row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
    let encoded = row.encode().unwrap();
    let (row_ref, _) = SpireLeafAssignmentRow::decode_prefix_ref(&encoded)
        .expect("row ref decodes");
    assert!(!is_visible_primary_assignment_ref(&row_ref));
}

#[test]
fn miri_is_delete_delta_assignment_flags_distinguishes_zero_and_set() {
    // Targets helpers.rs:69:5 (`-> true`), 69:11 (`&` -> `|`, `&` -> `^`).
    // - body -> true: flags=0 must return false; mutant returns true.
    // - `&` -> `|`: flags=0 → `0 | DELETE != 0` = true; original false.
    // - `&` -> `^`: flags=DELETE → `DELETE ^ DELETE = 0` != 0 = false;
    //   original `DELETE & DELETE = DELETE` != 0 = true.
    assert!(!super::is_delete_delta_assignment_flags(0));
    assert!(super::is_delete_delta_assignment_flags(
        SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
    ));
    // Sanity: TOMBSTONE alone is not a delete-delta.
    assert!(!super::is_delete_delta_assignment_flags(
        SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
    ));
}

#[test]
fn miri_validate_leaf_v2_locator_rejects_partial_max_block_or_offset() {
    // Targets helpers.rs:111:41 (`||` -> `&&`).
    // The check `block == u32::MAX || offset == u16::MAX` errors on
    // EITHER. The `&&` mutant requires BOTH, so a locator with only
    // block==MAX (and a valid offset) is accepted when it should
    // error.
    let only_block_max = ItemPointer {
        block_number: u32::MAX,
        offset_number: 1,
    };
    assert!(super::validate_leaf_v2_locator(only_block_max, "test").is_err());
    let only_offset_max = ItemPointer {
        block_number: 1,
        offset_number: u16::MAX,
    };
    assert!(super::validate_leaf_v2_locator(only_offset_max, "test").is_err());
    // Sanity: valid locator passes.
    let valid = ItemPointer {
        block_number: 1,
        offset_number: 1,
    };
    assert!(super::validate_leaf_v2_locator(valid, "test").is_ok());
}

#[test]
fn miri_decode_leaf_v2_local_vec_id_padding_check_starts_after_seq() {
    // Targets helpers.rs:179:16 (`+` -> `*`).
    // `input[1 + size_of::<u64>()..]` = `input[9..]` is the padding
    // window. Mutant `input[1 * 8..]` = `input[8..]` overlaps the
    // last byte of the seq.
    // With seq = 1 << 56, byte 8 of the little-endian sequence is
    // 0x01 (non-zero). Original: padding window (bytes 9-15) is all
    // zero → ok. Mutant: includes byte 8 = 0x01 → padding nonzero →
    // errors "padding must be zero".
    let mut input = vec![SPIRE_LOCAL_VEC_ID_DISCRIMINATOR];
    let seq: u64 = 1 << 56; // last byte of LE encoding is 0x01.
    input.extend_from_slice(&seq.to_le_bytes());
    input.extend_from_slice(&[0u8; LEAF_V2_LOCAL_VEC_ID_STRIDE - 9]);
    assert_eq!(input.len(), LEAF_V2_LOCAL_VEC_ID_STRIDE);
    let decoded = decode_leaf_v2_local_vec_id(&input)
        .expect("seq with high byte must decode cleanly");
    assert_eq!(decoded, seq);
}

#[test]
fn miri_leaf_v2_assignment_vec_id_layout_accepts_boundary_global_strides() {
    // Targets helpers.rs:233:23 (`<` -> `==`, `<` -> `<=`) and
    // 233:37 (`>` -> `==`, `>` -> `>=`). The check is
    // `stride < 2 || stride > MAX`. Stride=2 (min) and stride=MAX
    // must both pass original and fail various mutants.
    // (233:27 `||` -> `&&` is equivalent in practice — stride<2
    // is unreachable through SpireVecId::from_bytes' validation.)
    let min_global = SpireVecId::global(&[0x42]).unwrap();
    assert_eq!(min_global.as_bytes().len(), 2);
    let min_row = SpireLeafAssignmentRow {
        flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        vec_id: min_global,
        heap_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        gamma: 0.5,
        encoded_payload: vec![1, 2, 3, 4],
    };
    let (kind, stride) = super::leaf_v2_assignment_vec_id_layout(&min_row)
        .expect("stride=2 must be accepted at the lower boundary");
    assert_eq!(kind, SpireVecIdKind::GlobalBytes);
    assert_eq!(stride, 2);

    let max_payload = vec![0x77_u8; SPIRE_VEC_ID_MAX_BYTES - 1];
    let max_global = SpireVecId::global(&max_payload).unwrap();
    assert_eq!(max_global.as_bytes().len(), SPIRE_VEC_ID_MAX_BYTES);
    let max_row = SpireLeafAssignmentRow {
        flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        vec_id: max_global,
        heap_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        gamma: 0.25,
        encoded_payload: vec![1, 2, 3, 4],
    };
    let (kind, stride) = super::leaf_v2_assignment_vec_id_layout(&max_row)
        .expect("stride=MAX must be accepted at the upper boundary");
    assert_eq!(kind, SpireVecIdKind::GlobalBytes);
    assert_eq!(stride, SPIRE_VEC_ID_MAX_BYTES);
}

#[test]
fn miri_leaf_v2_max_segment_rows_returns_expected_count_and_errors_on_zero_room() {
    // Targets helpers.rs:324:34 (`-` -> `+`), 324:49 (`/` -> `*`, `/` -> `%`),
    // 325:16 (`>` -> `==`, `>` -> `<`, `>` -> `>=`),
    // 328:14 (`-=` -> `+=`, `-=` -> `/=`).
    //
    // For a known page configuration the function returns a specific
    // row count; arithmetic mutations either error out, infinite-loop
    // (caught by the 60-second test timeout), or return a wrong count.
    let rows =
        super::leaf_v2_max_segment_rows(8192, 4, super::LEAF_V2_LOCAL_VEC_ID_STRIDE)
            .expect("standard layout fits some rows");
    // PG page = 8192; usable ≈ 8168; segment fixed ≈ 72 → ~32 bytes
    // per row (2 flags + 16 vec_id + 6 tid + 4 gamma + 4 payload).
    // Expected rows = floor((8168 - 72) / 32) = 253.
    assert!(rows >= 200, "expected ~253 rows, got {rows}");
    assert!(rows <= 300, "expected ~253 rows, got {rows}");

    // Page too small to host even one row → error.
    let err = super::leaf_v2_max_segment_rows(64, 4, super::LEAF_V2_LOCAL_VEC_ID_STRIDE)
        .expect_err("a 64-byte page cannot fit a leaf V2 segment");
    assert!(
        err.contains("exceed page usable bytes") || err.contains("do not fit"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_validate_delta_assignment_requires_tombstone_on_delete() {
    // Targets helpers.rs:402:38 (`&` -> `|`, `&` -> `^`).
    // A delete-delta row WITHOUT TOMBSTONE must error.
    // - `&` -> `|`: `flags | TOMBSTONE` is always nonzero → mutant
    //   never enters the error branch.
    // - `&` -> `^`: `flags ^ TOMBSTONE == 0` only when flags ==
    //   TOMBSTONE; for flags=DELTA_DELETE alone, mutant sees nonzero
    //   and doesn't error.
    let row = SpireLeafAssignmentRow {
        flags: SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        vec_id: SpireVecId::local(1),
        heap_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
        gamma: 0.0,
        encoded_payload: vec![],
    };
    let err = super::validate_delta_assignment(&row)
        .expect_err("delete-delta without TOMBSTONE must be rejected");
    assert!(
        err.contains("must be tombstoned"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_validate_leaf_assignment_accepts_role_only_flag_combinations() {
    // Targets helpers.rs:439:9 (`|` -> `&`) and 440:9 (`|` -> `&`).
    // The role_flags constant is PRIMARY | BOUNDARY | TOMBSTONE |
    // STALE. Replacing any `|` with `&` collapses role_flags to 0
    // (disjoint bits), so the guard `flags & role_flags == 0` fires
    // for any flags value — the mutant rejects what the original
    // accepts.
    //
    // (The matching `|` -> `^` mutations at 438/439/440/444:58 are
    // equivalent — XOR of disjoint single-bit flags equals OR.)
    let mut row = leaf_v2_assignment(1, 4);
    row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
    super::validate_leaf_assignment(&row).expect("PRIMARY-only row is valid");
    row.flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
    super::validate_leaf_assignment(&row).expect("BOUNDARY-only row is valid");
}

#[test]
fn miri_is_delete_delta_assignment_wrapper_rejects_non_delete_rows() {
    // Targets helpers.rs:73:5 (`is_delete_delta_assignment -> bool with true`).
    // The wrapper at line 72-74 delegates to is_delete_delta_assignment_flags;
    // a non-delete row (PRIMARY only) must return false. With body→true the
    // wrapper always returns true, breaking the negative-case assertion.
    let mut row = leaf_v2_assignment(1, 4);
    row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
    assert!(!is_delete_delta_assignment(&row));
}

#[test]
fn miri_validate_leaf_v2_header_rejects_non_leaf_kind() {
    // Targets helpers.rs:80:5 (`validate_leaf_v2_header -> Ok(())`).
    // A header with a non-Leaf kind must surface the kind-mismatch error
    // through the leaf_v2 segment validation path; the body-replacement
    // mutant silently passes.
    let mut meta = leaf_v2_test_meta(1, 1);
    meta.header.assignment_count = 1;
    // Build a segment whose header is intentionally not a Leaf kind.
    let mut segment = leaf_v2_test_segment(
        &meta,
        0,
        0,
        &[leaf_v2_assignment(1, 4)],
        ItemPointer::INVALID,
    );
    segment.header.kind = SpirePartitionObjectKind::Internal;
    let object = super::SpireLeafPartitionObjectV2 {
        meta,
        segments: vec![segment],
    };
    let err = object
        .column_segments()
        .err()
        .expect("non-Leaf segment kind must be rejected via validate_leaf_v2_header");
    assert!(
        err.contains("kind must be Leaf") || err.contains("validate"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_validate_leaf_assignment_tombstone_only_skips_scored_payload_validation() {
    // Targets helpers.rs:444:25 (`&` -> `|`, `&` -> `^`).
    // The check `flags & (PRIMARY | BOUNDARY) != 0` decides whether
    // to invoke validate_scored_assignment_payload. Replacing `&`
    // with `|` or `^` makes the predicate always-true for any
    // flag pattern; a TOMBSTONE-only row with NONE payload would
    // pass the scored-payload validator (which requires
    // non-NONE format + non-empty payload), so the mutant errors
    // where the original accepts.
    let row = SpireLeafAssignmentRow {
        flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
        vec_id: SpireVecId::local(7),
        heap_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
        gamma: 0.0,
        encoded_payload: vec![],
    };
    super::validate_leaf_assignment(&row)
        .expect("tombstone-only with empty payload is valid");
}
