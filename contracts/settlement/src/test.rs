#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
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

#[test]
fn test_create_and_pay_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let governance_addr = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);

    let (asset_addr, sac_client, token_client) = create_token_contract(&env, &admin);
    sac_client.mint(&payer, &10_000_i128);

    client.initialize(&admin, &governance_addr);

    let payment_id = client.create_payment_request(&merchant, &1_000_i128, &asset_addr);
    assert_eq!(payment_id, 1);

    let payment = client.get_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);
    assert_eq!(payment.amount, 1_000);

    client.pay(&payer, &payment_id);

    let settled = client.get_payment(&payment_id);
    assert_eq!(settled.status, PaymentStatus::Settled);

    // 1.5% fee on 1000 = 15
    assert_eq!(token_client.balance(&merchant), 985);
    assert_eq!(token_client.balance(&admin), 15);
    assert_eq!(token_client.balance(&payer), 9_000);
}

#[test]
fn test_cannot_pay_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let governance_addr = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

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
    let governance_addr = Address::generate(&env);
    let merchant = Address::generate(&env);

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
    let governance_addr = Address::generate(&env);
    let merchant = Address::generate(&env);

    let contract_id = env.register_contract(None, AjoPaySettlement);
    let client = AjoPaySettlementClient::new(&env, &contract_id);
    let (asset_addr, _sac, _tok) = create_token_contract(&env, &admin);

    client.initialize(&admin, &governance_addr);
    let result = client.try_create_payment_request(&merchant, &0_i128, &asset_addr);
    assert!(result.is_err());
}
