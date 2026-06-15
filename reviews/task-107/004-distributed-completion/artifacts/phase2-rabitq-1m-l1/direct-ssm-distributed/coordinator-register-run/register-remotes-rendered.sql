\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('task107_phase2_rabitq_1m_l1_idx'::regclass::oid, 2, 1, 'ecaz-spire-aws-aa606602-remote-1-20260614203301856800000002', decode('a7b56a9cf0d817f9', 'hex'), 'task107_phase2_rabitq_1m_l1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('task107_phase2_rabitq_1m_l1_idx'::regclass::oid, 3, 1, 'ecaz-spire-aws-aa606602-remote-2-20260614203301857100000006', decode('9e86b55f582864a2', 'hex'), 'task107_phase2_rabitq_1m_l1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

