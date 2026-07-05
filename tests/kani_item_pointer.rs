#![cfg(kani)]

use ecaz::bench_api::ItemPointer;

#[kani::proof]
fn kani_item_pointer_decode_contract() {
    let input: [u8; 7] = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= input.len());

    let result = ItemPointer::decode(&input[..len]);
    if len == ecaz::bench_api::ITEM_POINTER_BYTES {
        assert!(result.is_ok());
    } else {
        assert!(result.is_err());
    }
}
