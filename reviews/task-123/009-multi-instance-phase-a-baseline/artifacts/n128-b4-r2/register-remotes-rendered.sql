\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n128_b4_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('bd7826dc48d00047', 'hex'), 't123_p9_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n128_b4_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('58b195b90dc8c0b0', 'hex'), 't123_p9_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n128_b4_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('a8891b3a43f7261a', 'hex'), 't123_p9_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

