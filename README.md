# AjoPay-Contract

Soroban smart contracts powering **AjoPay** — a non-custodial merchant payment and
settlement platform on Stellar, built for Nigerian and African merchants.

> "Ajo" is the traditional rotating community savings system found across Nigeria.
> AjoPay brings that same trust-based, community-rooted approach to on-chain,
> dollar-denominated (USDC) merchant payments — settled directly into local fiat.

## Contracts

| Contract | Purpose |
|----------|---------|
| `settlement` | Handles payment requests, payment execution, and fee splitting between merchant, platform, and anchor |
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
| Settlement | `TBD — fill in after running deploy_testnet.sh` |
| Governance | `TBD — fill in after running deploy_testnet.sh` |
| Admin | `TBD` |

Network: Test SDF Network

## Design Notes

- **Fee splitting** happens atomically inside `settlement::pay` — merchant, platform,
  and anchor shares are computed from a basis-point config pulled from `governance`.
- **Anchor registry** in `governance` lets the platform whitelist which fiat off-ramp
  providers (NGN, KES, GHS, etc.) a merchant can settle into, without redeploying the
  settlement contract.
- Contracts are intentionally kept small and single-purpose so each can be audited
  and upgraded independently.

## License

MIT