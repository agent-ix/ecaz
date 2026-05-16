#[allow(dead_code)]
#[path = "../../../src/storage/page.rs"]
mod page;

#[cfg(kani)]
#[kani::proof]
fn kani_item_pointer_decode_contract() {
    let block_number: u32 = kani::any();
    let offset_number: u16 = kani::any();
    let pointer = page::ItemPointer {
        block_number,
        offset_number,
    };

    let mut encoded = Vec::new();
    pointer.encode_into(&mut encoded);
    let decoded = page::ItemPointer::decode(&encoded).unwrap();

    assert_eq!(decoded.block_number, block_number);
    assert_eq!(decoded.offset_number, offset_number);
}
