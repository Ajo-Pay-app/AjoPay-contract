#![cfg(test)]

use super::*;
use ajopay_governance::{AjoPayGovernance, AjoPayGovernanceClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, token::StellarAssetClient<'a>, token::Client<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    let sac_client = token::StellarAssetClient::new(env, &contract_address.address());
    let token_client = token::Client::new(env, &contract_address.address());
    (contract_address.address(), sac_client, token_client)
}

/// Deploys and initializes a real governance contract with the given fee,
/// so settlement's cross-contract `platform_fee_bps` call has something
/// real to talk to — this is the Day 6 integration, not a mock.
fn deploy_governance(env: &Env, admin: &Address, fee_bps: u32) -> Address {
    let governance_id = env.register_contract(None, AjoPayGovernance);
    let governance_client = AjoPayGovernanceClient::new(env, &governance_id);
    governance_client.initialize(admin, &fee_bps);
    governance_id
}

#[test]
fn test_create_and_pay_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let governance_addr = deploy_governance(&env, &admin, 150); // 1.5%

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);

    let (asset_addr, sac_client, token_client) = create_token_contract(&env, &admin);
    sac_client.mint(&payer, &10_000_i128);

    client.initialize(&admin, &governance_addr);

    let payment_id = client.create_payment_request(&merchant, &1_000_i128, &asset_addr);
    assert_eq!(payment_id, 1);

    client.pay(&payer, &payment_id);

    let settled = client.get_payment(&payment_id);
    assert_eq!(settled.status, PaymentStatus::Settled);

    // 1.5% fee on 1000 = 15 — this value now comes from the governance call,
    // not a hardcoded constant in settlement.
    assert_eq!(token_client.balance(&merchant), 985);
    assert_eq!(token_client.balance(&admin), 15);
    assert_eq!(token_client.balance(&payer), 9_000);
}

#[test]
fn test_fee_changes_take_effect_live() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let governance_id = env.register_contract(None, AjoPayGovernance);
    let governance_client = AjoPayGovernanceClient::new(&env, &governance_id);
    governance_client.initialize(&admin, &100u32); // 1.0%

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);

    let (asset_addr, sac_client, token_client) = create_token_contract(&env, &admin);
    sac_client.mint(&payer, &20_000_i128);

    client.initialize(&admin, &governance_id);

    // First payment at 1.0%.
    let p1 = client.create_payment_request(&merchant, &1_000_i128, &asset_addr);
    client.pay(&payer, &p1);
    assert_eq!(token_client.balance(&admin), 10); // 1% of 1000

    // Admin raises the fee on governance — settlement should pick it up
    // on the very next payment with no redeploy.
    governance_client.set_fee_bps(&admin, &300u32); // 3.0%

    let p2 = client.create_payment_request(&merchant, &1_000_i128, &asset_addr);
    client.pay(&payer, &p2);
    assert_eq!(token_client.balance(&admin), 10 + 30); // +3% of the second 1000
}

#[test]
fn test_cannot_pay_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let governance_addr = deploy_governance(&env, &admin, 150);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);

    let (asset_addr, sac_client, _token_client) = create_token_contract(&env, &admin);
    sac_client.mint(&payer, &10_000_i128);

    client.initialize(&admin, &governance_addr);
    let payment_id = client.create_payment_request(&merchant, &500_i128, &asset_addr);

    client.pay(&payer, &payment_id);

    let result = client.try_pay(&payer, &payment_id);
    assert!(result.is_err());
}

#[test]
fn test_merchant_can_cancel_pending_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);

    let governance_addr = deploy_governance(&env, &admin, 150);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);
    let (asset_addr, _sac, _tok) = create_token_contract(&env, &admin);

    client.initialize(&admin, &governance_addr);
    let payment_id = client.create_payment_request(&merchant, &200_i128, &asset_addr);

    client.cancel(&merchant, &payment_id);

    let payment = client.get_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Cancelled);
}

#[test]
fn test_rejects_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);

    let governance_addr = deploy_governance(&env, &admin, 150);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);
    let (asset_addr, _sac, _tok) = create_token_contract(&env, &admin);

    client.initialize(&admin, &governance_addr);
    let result = client.try_create_payment_request(&merchant, &0_i128, &asset_addr);
    assert!(result.is_err());
}

#[test]
fn test_non_admin_cannot_repoint_governance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let governance_addr = deploy_governance(&env, &admin, 150);
    let other_governance_addr = deploy_governance(&env, &admin, 500);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);

    client.initialize(&admin, &governance_addr);

    let result = client.try_set_governance_addr(&stranger, &other_governance_addr);
    assert!(result.is_err());

    client.set_governance_addr(&admin, &other_governance_addr);
    assert_eq!(client.get_governance_addr(), other_governance_addr);
}
