#[allow(dead_code)]
#[path = "../../../src/storage/page.rs"]
mod page;

#[cfg(test)]
mod tests {
    use super::page::{DataPageChain, ItemPointer, FIRST_DATA_BLOCK_NUMBER};

    #[test]
    fn item_pointer_decode_rejects_short_payloads() {
        assert!(ItemPointer::decode(&[0; 5]).is_err());
    }

    #[test]
    fn page_chain_preserves_payloads_across_overflow() {
        let mut chain = DataPageChain::new(128);
        let first = chain.insert_raw_tuple(vec![1; 32]).unwrap();
        let second = chain.insert_raw_tuple(vec![2; 32]).unwrap();
        let third = chain.insert_raw_tuple(vec![3; 32]).unwrap();

        assert_eq!(first.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(second.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(third.block_number, FIRST_DATA_BLOCK_NUMBER + 1);
        assert_eq!(
            chain
                .get_page(third.block_number)
                .unwrap()
                .raw_tuple(third)
                .unwrap(),
            &[3; 32]
        );
    }
}
