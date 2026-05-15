    #[pg_test]
    #[should_panic(expected = "canonical default")]
    fn test_ech_insert_rejects_mismatched_seed() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_seed_mismatch (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_seed_mismatch VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_seed_mismatch_idx ON ec_hnsw_insert_seed_mismatch USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_seed_mismatch VALUES
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 43))",
        )
        .expect("insert should fail");
    }

    #[pg_test]
    #[should_panic(expected = "code length mismatch")]
    fn test_binary_recv_rejects_trailing_bytes() {
        let mut bytes = pack(4, 4, 42, 0.5, &[0x11, 0x22, 0x33]);
        bytes.push(0x44);
        let _ = unsafe { recv_via_string_info(&bytes) };
    }

    #[pg_test]
    #[should_panic(expected = "too short")]
    fn test_binary_recv_rejects_truncated_bytes() {
        let _ = unsafe { recv_via_string_info(&[0_u8; 5]) };
    }

    unsafe fn recv_via_string_info(bytes: &[u8]) -> Vec<u8> {
        let leaked = Box::leak(bytes.to_vec().into_boxed_slice());
        let raw = pg_sys::StringInfoData {
            data: leaked.as_mut_ptr() as *mut std::os::raw::c_char,
            len: leaked.len() as i32,
            maxlen: leaked.len() as i32,
            cursor: 0,
        };
        tqvector_recv(Internal::new(raw))
    }
