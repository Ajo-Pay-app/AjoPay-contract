#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String};

#[test]
fn test_version() {
    let env = Env::default();
    let contract_id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &contract_id);

    let version = client.version();
    assert_eq!(version, String::from_str(&env, "0.1.0"));
}
