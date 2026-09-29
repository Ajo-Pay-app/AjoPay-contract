#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Env, Symbol,
};

/// ---------------------------------------------------------------------
/// Data model
/// ---------------------------------------------------------------------

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    GovernanceAddr,
    PaymentCounter,
    Payment(u64),
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum PaymentStatus {
    Pending,
    Settled,
    Cancelled,
}

#[derive(Clone)]
#[contracttype]
pub struct Payment {
    pub merchant: Address,
    pub payer: Option<Address>,
    pub amount: i128,
    pub asset: Address,
    pub status: PaymentStatus,
    pub created_at: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    PaymentNotFound = 3,
    PaymentNotPending = 4,
    Unauthorized = 5,
    InvalidAmount = 6,
}

const PLATFORM_FEE_BPS: i128 = 150; // 1.5% default, overridable via governance later

/// ---------------------------------------------------------------------
/// Contract
/// ---------------------------------------------------------------------

#[contract]
pub struct AjoPaySettlement;

#[contractimpl]
impl AjoPaySettlement {
    /// One-time setup. `admin` can configure the contract later;
    /// `governance_addr` is the deployed governance contract this settlement
    /// contract will eventually read fee/anchor config from.
    pub fn initialize(env: Env, admin: Address, governance_addr: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::GovernanceAddr, &governance_addr);
        env.storage().instance().set(&DataKey::PaymentCounter, &0u64);
        Ok(())
    }

    /// Merchant creates a payment request for `amount` of `asset` (e.g. USDC SAC address).
    /// Returns the new payment id.
    pub fn create_payment_request(
        env: Env,
        merchant: Address,
        amount: i128,
        asset: Address,
    ) -> Result<u64, Error> {
        merchant.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentCounter)
            .ok_or(Error::NotInitialized)?;
        counter += 1;

        let payment = Payment {
            merchant: merchant.clone(),
            payer: None,
            amount,
            asset,
            status: PaymentStatus::Pending,
            created_at: env.ledger().timestamp(),
        };

        env.storage().instance().set(&DataKey::PaymentCounter, &counter);
        env.storage()
            .persistent()
            .set(&DataKey::Payment(counter), &payment);

        env.events().publish(
            (Symbol::new(&env, "payment_created"), merchant),
            (counter, amount),
        );

        Ok(counter)
    }

    /// Payer settles a pending payment. Transfers `amount` of `asset` from `payer`
    /// to this contract, then splits it out: merchant gets (amount - platform fee),
    /// platform (this contract's admin) gets the fee.
    ///
    /// NOTE: for a production build you'd route the platform-fee share to a
    /// configurable treasury address (read from governance) rather than the admin
    /// directly — left as a Week 2 governance-integration task, see AjoPay-Contract
    /// README "Design Notes".
    pub fn pay(env: Env, payer: Address, payment_id: u64) -> Result<(), Error> {
        payer.require_auth();

        let mut payment: Payment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .ok_or(Error::PaymentNotFound)?;

        if payment.status != PaymentStatus::Pending {
            return Err(Error::PaymentNotPending);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        let token_client = token::Client::new(&env, &payment.asset);

        let fee = (payment.amount * PLATFORM_FEE_BPS) / 10_000;
        let merchant_share = payment.amount - fee;

        // Pull full amount from payer into this contract, then disburse.
        token_client.transfer(&payer, &env.current_contract_address(), &payment.amount);
        token_client.transfer(&env.current_contract_address(), &payment.merchant, &merchant_share);
        if fee > 0 {
            token_client.transfer(&env.current_contract_address(), &admin, &fee);
        }

        payment.payer = Some(payer.clone());
        payment.status = PaymentStatus::Settled;
        env.storage()
            .persistent()
            .set(&DataKey::Payment(payment_id), &payment);

        env.events().publish(
            (Symbol::new(&env, "payment_settled"), payment.merchant.clone()),
            (payment_id, merchant_share, fee),
        );

        Ok(())
    }

    /// Merchant or admin cancels a pending (unpaid) payment request.
    pub fn cancel(env: Env, caller: Address, payment_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut payment: Payment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .ok_or(Error::PaymentNotFound)?;

        if payment.status != PaymentStatus::Pending {
            return Err(Error::PaymentNotPending);
        }
        if caller != payment.merchant {
            return Err(Error::Unauthorized);
        }

        payment.status = PaymentStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Payment(payment_id), &payment);
        Ok(())
    }

    pub fn get_payment(env: Env, payment_id: u64) -> Result<Payment, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .ok_or(Error::PaymentNotFound)
    }
}

mod test;
