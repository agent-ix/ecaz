    #[pg_test]
    fn test_ec_spire_access_method_is_registered() {
        let amname =
            Spi::get_one::<String>("SELECT amname::text FROM pg_am WHERE amname = 'ec_spire'")
                .expect("SPI query should succeed")
                .expect("access method should exist");
        assert_eq!(amname, "ec_spire");
    }

    #[pg_test]
    fn test_ec_spire_operator_classes_are_registered() {
        let opclasses = Spi::get_one::<i64>(
            "SELECT count(*) FROM pg_opclass opc \
             JOIN pg_am am ON am.oid = opc.opcmethod \
             WHERE am.amname = 'ec_spire' \
             AND opc.opcname IN ('tqvector_spire_ip_ops', 'ecvector_spire_ip_ops')",
        )
        .expect("SPI query should succeed")
        .expect("operator class count should exist");
        assert_eq!(opclasses, 2);
    }

    #[pg_test]
    fn test_ec_spire_custom_scan_status_registered_fail_closed() {
        let status_from = "FROM ec_spire_custom_scan_status()";
        let provider_name = Spi::get_one::<String>(&format!("SELECT provider_name {status_from}"))
            .expect("custom scan provider name query should succeed")
            .expect("custom scan provider name should exist");
        let registered = Spi::get_one::<bool>(&format!("SELECT registered {status_from}"))
            .expect("custom scan registered query should succeed")
            .expect("custom scan registered value should exist");
        let hook_installed =
            Spi::get_one::<bool>(&format!("SELECT rel_pathlist_hook_installed {status_from}"))
                .expect("custom scan hook query should succeed")
                .expect("custom scan hook value should exist");
        let path_generation_enabled =
            Spi::get_one::<bool>(&format!("SELECT path_generation_enabled {status_from}"))
                .expect("custom scan path generation query should succeed")
                .expect("custom scan path generation value should exist");
        let exec_wiring_enabled =
            Spi::get_one::<bool>(&format!("SELECT exec_wiring_enabled {status_from}"))
                .expect("custom scan exec wiring query should succeed")
                .expect("custom scan exec wiring value should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {status_from}"))
            .expect("custom scan status query should succeed")
            .expect("custom scan status should exist");

        assert_eq!(provider_name, "EcSpireDistributedScan");
        assert!(registered);
        assert!(hook_installed);
        assert!(path_generation_enabled);
        assert!(exec_wiring_enabled);
        assert_eq!(status, "executor_stream_wired_tuple_payload_slots");
    }
