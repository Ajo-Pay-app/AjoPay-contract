#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    PlatformFeeBps,
    Anchor(Address), // -> bool trusted flag
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidFee = 4,
}

#[contract]
pub struct AjoPayGovernance;

#[contractimpl]
impl AjoPayGovernance {
    pub fn initialize(env: Env, admin: Address, initial_fee_bps: u32) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        if initial_fee_bps > 10_000 {
            return Err(Error::InvalidFee);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &initial_fee_bps);
        Ok(())
    }

    /// Admin-only: update the platform fee (in basis points, e.g. 150 = 1.5%).
    pub fn set_fee_bps(env: Env, caller: Address, new_fee_bps: u32) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        if new_fee_bps > 10_000 {
            return Err(Error::InvalidFee);
        }
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &new_fee_bps);
        Ok(())
    }

    pub fn get_fee_bps(env: Env) -> Result<u32, Error> {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .ok_or(Error::NotInitialized)
    }

    /// Admin-only: whitelist or de-whitelist a Stellar Anchor (fiat off-ramp)
    /// by its contract/account address.
    pub fn set_anchor_trusted(
        env: Env,
        caller: Address,
        anchor: Address,
        trusted: bool,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::Anchor(anchor), &trusted);
        Ok(())
    }

    pub fn is_anchor_trusted(env: Env, anchor: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Anchor(anchor))
            .unwrap_or(false)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

mod test;
