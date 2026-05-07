    #[test]
    fn local_vec_id_round_trips_sequence() {
        let vec_id = SpireVecId::local(42);

        assert_eq!(vec_id.discriminator(), SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(vec_id.local_sequence(), Some(42));
        assert_eq!(
            SpireVecId::from_bytes(vec_id.as_bytes())
                .unwrap()
                .local_sequence(),
            Some(42)
        );
    }

    #[test]
    fn vec_id_rejects_invalid_shapes() {
        assert!(SpireVecId::from_bytes(&[]).is_err());
        assert!(SpireVecId::from_bytes(&[0xff, 1, 2]).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 1]).is_err());
        assert!(SpireVecId::from_bytes(SpireVecId::local(0).as_bytes()).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR]).is_err());
        assert!(SpireVecId::global(&vec![7; SPIRE_VEC_ID_MAX_BYTES]).is_err());
    }

    #[test]
    fn global_vec_id_preserves_payload() {
        let vec_id = SpireVecId::global(&[9, 8, 7]).unwrap();

        assert_eq!(vec_id.discriminator(), SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(
            vec_id.as_bytes(),
            &[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, 9, 8, 7]
        );
        assert_eq!(SpireVecId::from_bytes(vec_id.as_bytes()).unwrap(), vec_id);
    }

    #[test]
    fn partition_object_header_decodes_prefix_and_payload_tail() {
        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 1,
            parent_pid: 5,
            child_count: 0,
            assignment_count: 99,
            flags: 0x10,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&[1, 2, 3]);

        let (decoded, tail) = SpirePartitionObjectHeader::decode_prefix(&encoded).unwrap();

        assert_eq!(decoded, header);
        assert_eq!(tail, &[1, 2, 3]);
    }

    #[test]
    fn partition_object_header_rejects_invalid_identity() {
        let mut header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Internal,
            pid: 0,
            object_version: 1,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 1,
            assignment_count: 0,
            flags: 0,
        };
        assert!(header.encode().is_err());
        header.pid = 1;
        header.object_version = 0;
        assert!(header.encode().is_err());
    }

    #[test]
    fn partition_object_constructors_reject_invalid_header_identity() {
        let row = leaf_v2_assignment(1, 8);

        assert!(SpireLeafPartitionObject::new(0, 3, 0, vec![row.clone()]).is_err());
        assert!(SpireLeafPartitionObject::new(17, 0, 0, vec![row]).is_err());
        assert!(SpireDeltaPartitionObject::new(0, 4, 17, Vec::new()).is_err());
        assert!(SpireDeltaPartitionObject::new(19, 0, 17, Vec::new()).is_err());
        assert!(SpireRoutingPartitionObject::root(0, 3, 2, routing_children()).is_err());
        assert!(SpireRoutingPartitionObject::root(11, 0, 2, routing_children()).is_err());
    }

    #[test]
    fn routing_partition_object_round_trips_root_children() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(decoded.header.level, 1);
        assert_eq!(decoded.header.parent_pid, 0);
        assert_eq!(decoded.header.child_count, 2);
        assert_eq!(decoded.header.assignment_count, 0);
        assert_eq!(decoded.child_pids[0], 17);
        assert_eq!(decoded.child_centroid(1).unwrap(), &[-1.0, 0.0]);
    }

    #[test]
    fn routing_partition_object_round_trips_internal_children() {
        let object =
            SpireRoutingPartitionObject::internal(12, 4, 2, 11, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Internal);
        assert_eq!(decoded.header.level, 2);
        assert_eq!(decoded.header.parent_pid, 11);
    }

    #[test]
    fn local_store_relation_name_is_deterministic() {
        assert_eq!(
            spire_local_store_relation_name(12345, 0).unwrap(),
            "ec_spire_store_12345_0"
        );
        assert_eq!(
            spire_local_store_relation_name(u32::MAX, 16).unwrap(),
            "ec_spire_store_4294967295_16"
        );
        assert!(spire_local_store_relation_name(0, 0).is_err());
    }
