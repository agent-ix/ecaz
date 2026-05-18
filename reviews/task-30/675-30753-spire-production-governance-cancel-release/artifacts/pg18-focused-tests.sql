\set ON_ERROR_STOP on
CREATE EXTENSION IF NOT EXISTS ecaz CASCADE;
SELECT tests."test_ec_spire_production_fault_matrix_contract"();
SELECT tests."test_ec_spire_prod_transport_governance_overload"();
SELECT tests."test_ec_spire_prod_receive_governance_overload"();
SELECT tests."test_ec_spire_prod_transport_local_cancel_remote_cancel"();
SELECT tests."test_ec_spire_prod_receive_local_cancel_remote_cancel"();
