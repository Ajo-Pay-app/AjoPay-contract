#![no_std]
use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn version(env: Env) -> String {
        String::from_str(&env, "0.1.0")
    }
}

mod test;
