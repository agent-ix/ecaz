//! Layout stability assertions for tqvector storage formats.
//! Ensures that payload sizes, struct sizes, and wire formats
//! remain stable across code changes.

use tqvector::bench_api::{
    mse_code_len, payload_len, qjl_code_len, ItemPointer, TqElementTuple, HEAPTID_INLINE_CAPACITY,
    ITEM_POINTER_BYTES, PAGE_HEADER_BYTES,
};

// --- NFR-002 payload size contracts ---

#[test]
fn payload_len_1536_dim_4bit() {
    // 4 gamma + 576 MSE + 192 QJL = 772
    assert_eq!(payload_len(1536, 4), 772);
}

#[test]
fn payload_len_1536_dim_2bit() {
    // mse: (1536 * 1) / 8 = 192, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 2), 192);
    assert_eq!(payload_len(1536, 2), 4 + 192 + 192);
}

#[test]
fn payload_len_1536_dim_3bit() {
    // mse: (1536 * 2).div_ceil(8) = 384, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 3), 384);
    assert_eq!(payload_len(1536, 3), 4 + 384 + 192);
}

#[test]
fn payload_len_1536_dim_6bit() {
    // mse: (1536 * 5).div_ceil(8) = 960, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 6), 960);
    assert_eq!(payload_len(1536, 6), 4 + 960 + 192);
}

#[test]
fn payload_len_1536_dim_8bit() {
    // mse: (1536 * 7).div_ceil(8) = 1344, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 8), 1344);
    assert_eq!(payload_len(1536, 8), 4 + 1344 + 192);
}

#[test]
fn mse_code_len_1536_4bit() {
    // 1536 * 3 bits / 8 = 576
    assert_eq!(mse_code_len(1536, 4), 576);
}

#[test]
fn qjl_code_len_1536() {
    // 1536 / 8 = 192
    assert_eq!(qjl_code_len(1536), 192);
}

// --- Struct sizes ---

#[test]
fn item_pointer_struct_size() {
    assert_eq!(std::mem::size_of::<ItemPointer>(), 8); // u32 + u16 + padding
}

#[test]
fn item_pointer_wire_size() {
    assert_eq!(ITEM_POINTER_BYTES, 6);
}

#[test]
fn page_header_size() {
    assert_eq!(PAGE_HEADER_BYTES, 24);
}

#[test]
fn heaptid_inline_capacity() {
    assert_eq!(HEAPTID_INLINE_CAPACITY, 10);
}

// --- Element tuple encoded length ---

#[test]
fn element_tuple_encoded_len_1536_4bit() {
    let code_len = mse_code_len(1536, 4) + qjl_code_len(1536); // 576 + 192 = 768
    assert_eq!(code_len, 768);

    // tag(1) + level(1) + deleted(1) + 10*ItemPointer(60) + count(1) + gamma(4) + neighbortid(6) + code(768)
    let expected = 1
        + 1
        + 1
        + (HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES)
        + 1
        + 4
        + ITEM_POINTER_BYTES
        + code_len;
    assert_eq!(expected, 842);
    assert_eq!(TqElementTuple::encoded_len(code_len), expected);
}

// --- Compression ratio contract ---

#[test]
fn compression_ratio_1536_4bit() {
    let raw_fp32 = 1536 * 4; // 6144
    let tqvector_datum = 11 + payload_len(1536, 4); // header + payload = 11 + 772 = 783
    let ratio = raw_fp32 as f64 / tqvector_datum as f64;
    assert!(
        ratio >= 7.8,
        "compression ratio = {ratio:.2}x, expected >= 7.8x"
    );
}
