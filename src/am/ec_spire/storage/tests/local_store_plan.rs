    #[test]
    fn local_store_relation_plan_sorts_and_preserves_tablespaces() {
        let plan = plan_local_store_relations(12345, [(1, 10), (0, 10), (2, 11)]).unwrap();

        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].relation_name, "ec_spire_store_12345_0");
        assert_eq!(plan[0].tablespace_oid, 10);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 10);
        assert_eq!(plan[2].local_store_id, 2);
        assert_eq!(plan[2].tablespace_oid, 11);

        assert!(plan_local_store_relations(12345, []).is_err());
        assert!(plan_local_store_relations(12345, [(0, 10), (0, 11)]).is_err());
    }

    #[test]
    fn local_store_relation_plan_builds_store_config_from_created_relids() {
        let relation_plan = plan_local_store_relations(12345, [(1, 10), (0, 10), (2, 11)]).unwrap();

        let config = local_store_config_from_relation_plan(
            7,
            &relation_plan,
            [(2, 502), (0, 500), (1, 501)],
        )
        .unwrap();

        assert_eq!(config.generation, 7);
        assert_eq!(config.stores.len(), 3);
        assert_eq!(config.stores[0].local_store_id, 0);
        assert_eq!(config.stores[0].store_relid, 500);
        assert_eq!(config.stores[0].tablespace_oid, 10);
        assert_eq!(config.stores[1].local_store_id, 1);
        assert_eq!(config.stores[1].store_relid, 501);
        assert_eq!(config.stores[1].tablespace_oid, 10);
        assert_eq!(config.stores[2].local_store_id, 2);
        assert_eq!(config.stores[2].store_relid, 502);
        assert_eq!(config.stores[2].tablespace_oid, 11);

        assert!(
            local_store_config_from_relation_plan(7, &relation_plan, [(0, 500), (1, 501)]).is_err()
        );
        assert!(local_store_config_from_relation_plan(
            7,
            &relation_plan,
            [(0, 500), (1, 501), (2, 502), (3, 503)]
        )
        .is_err());
        assert!(local_store_config_from_relation_plan(
            7,
            &relation_plan,
            [(0, 500), (1, 501), (1, 511), (2, 502)]
        )
        .is_err());
    }

    #[test]
    fn local_object_store_preserves_descriptor_store_id() {
        let descriptor = SpireLocalStoreDescriptor::available(2, 12345, 67890).unwrap();
        let mut store = SpireLocalObjectStore::for_store_descriptor(&descriptor, 8192).unwrap();
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();

        let placement = store.insert_routing_object(7, &object).unwrap();

        assert_eq!(placement.local_store_id, 2);
        assert_eq!(placement.store_relid, 12345);
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;
        assert_eq!(store.read_routing_object(&placement).unwrap(), expected);

        let mut wrong_store_placement = placement;
        wrong_store_placement.local_store_id = 1;
        assert!(store.read_routing_object(&wrong_store_placement).is_err());
    }

    #[test]
    fn local_object_store_rejects_unavailable_descriptor() {
        let descriptor = SpireLocalStoreDescriptor {
            local_store_id: 2,
            store_relid: 12345,
            tablespace_oid: 67890,
            state: SpireLocalStoreState::Unavailable,
        };

        assert!(SpireLocalObjectStore::for_store_descriptor(&descriptor, 8192).is_err());
    }

