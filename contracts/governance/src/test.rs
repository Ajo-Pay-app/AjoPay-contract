#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

#[test]
fn test_initialize_and_get_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AjoPayGovernance);
    let client = AjoPayGovernanceClient::new(&env, &contract_id);

    client.initialize(&admin, &150u32);
    assert_eq!(client.get_fee_bps(), 150);
}

#[test]
fn test_admin_can_update_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AjoPayGovernance);
    let client = AjoPayGovernanceClient::new(&env, &contract_id);

    client.initialize(&admin, &150u32);
    client.set_fee_bps(&admin, &200u32);
    assert_eq!(client.get_fee_bps(), 200);
}

#[test]
fn test_non_admin_cannot_update_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let contract_id = env.register_contract(None, AjoPayGovernance);
    let client = AjoPayGovernanceClient::new(&env, &contract_id);

    client.initialize(&admin, &150u32);
    let result = client.try_set_fee_bps(&stranger, &500u32);
    assert!(result.is_err());
}

#[test]
fn test_anchor_registry() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let anchor = Address::generate(&env);
    let contract_id = env.register_contract(None, AjoPayGovernance);
    let client = AjoPayGovernanceClient::new(&env, &contract_id);

    client.initialize(&admin, &150u32);
    assert_eq!(client.is_anchor_trusted(&anchor), false);

    client.set_anchor_trusted(&admin, &anchor, &true);
    assert_eq!(client.is_anchor_trusted(&anchor), true);

    client.set_anchor_trusted(&admin, &anchor, &false);
    assert_eq!(client.is_anchor_trusted(&anchor), false);
}
