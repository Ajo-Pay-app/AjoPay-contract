# AjoPay-Contract

Soroban smart contracts powering **AjoPay** — a non-custodial merchant payment and
settlement platform on Stellar, built for Nigerian and African merchants.

> "Ajo" is the traditional rotating community savings system found across Nigeria.
> AjoPay brings that same trust-based, community-rooted approach to on-chain,
> dollar-denominated (USDC) merchant payments — settled directly into local fiat.

## Contracts

| Contract | Purpose |
|----------|---------|
| `settlement` | Handles payment requests, payment execution, and fee splitting between merchant and platform. Reads the live fee rate from `governance` on every payment via a cross-contract call, and exposes `set_governance_addr`/`get_governance_addr`/`get_admin` for admin management. |
| `governance` | Manages platform fee configuration and the registry of trusted Stellar Anchors (fiat off-ramps) |


## Project Structure

```
AjoPay-Contract/
├── Cargo.toml                  # workspace manifest
├── contracts/
│   ├── settlement/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── test.rs
│   └── governance/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── test.rs
└── scripts/
    └── deploy_testnet.sh
```

## Requirements

- Rust (stable) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli)
- `soroban-sdk` (pin the version to whatever is current when you set this up —
  check the [Soroban docs](https://developers.stellar.org/docs/build/smart-contracts)
  rather than assuming a fixed version)

## Quick Start

```bash
# Run all unit tests
cargo test

# Build both contracts to wasm
cargo build --target wasm32-unknown-unknown --release

# Deploy to Stellar testnet
bash scripts/deploy_testnet.sh
```

## Deployed Contract Addresses (Testnet)

| Contract | Address |
|----------|---------|
| Settlement | `CALRYY6HQDHT2E3KRI4S5ORW36WEI6GZ4IAFJRJKUN5IZAYXCDKAD6G3` |
| Governance | `CDSVLXSWLR4K7K6RLA6PYZECDSLYBXM6XFG5NRNKMAHSD4XLTD7EMB4H` |
| Admin | ` GAM7XIXTYY63CIZSG4IEIKUSJDWRXFJR3NTBPTOBZIAH66T5TSU2SV5X` |

Network: Test SDF Network

## Design Notes

- **Fee splitting** happens atomically inside `settlement::pay`. The fee rate is read
  live via a cross-contract call to `governance::get_fee_bps` on every payment, so an
  admin can change pricing by calling `governance.set_fee_bps` without redeploying
  settlement at all. See `test_fee_changes_take_effect_live` in
  `contracts/settlement/src/test.rs` for a working example of this.
- The fee itself still lands on settlement's own `admin` address rather than a
  separate treasury account — splitting those two roles apart is a reasonable next
  feature if you want fee collection and contract administration to be different
  parties.
- **Anchor registry** in `governance` lets the platform whitelist which fiat off-ramp
  providers (NGN, KES, GHS, etc.) a merchant can settle into, without redeploying the
  settlement contract.
- Contracts are intentionally kept small and single-purpose so each can be audited
  and upgraded independently.

## License

MIT
