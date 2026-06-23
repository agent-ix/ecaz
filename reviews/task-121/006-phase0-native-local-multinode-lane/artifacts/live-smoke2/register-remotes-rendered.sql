\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('task121_native_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('879195bd721fb81e', 'hex'), 'task121_native_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('task121_native_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('78b75176822b5c4b', 'hex'), 'task121_native_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('task121_native_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('44587b766464c57b', 'hex'), 'task121_native_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

