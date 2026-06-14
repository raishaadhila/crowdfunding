# Solana Crowdfunding Platform

A trustless crowdfunding smart contract on Solana. Think Kickstarter, but on-chain. Creators launch campaigns with a goal and deadline; donors contribute SOL into a PDA vault; funds are released to the creator only if the goal is met after the deadline, otherwise donors can reclaim their contributions.

## Overview

| | |
|---|---|
| **Program ID** | `Fs4wNeXAJKypKodn4kbDs7nmdJeFB6emVY2Ntj3FLqzt` |
| **Network** | Devnet |
| **Framework** | Anchor 1.0 |
| **Language** | Rust 2021 edition |

## How It Works

1. **Creator** launches a campaign with a SOL goal and deadline
2. **Donors** contribute SOL into a PDA vault
3. After the deadline:
   - If goal **met** → creator withdraws funds
   - If goal **not met** → donors claim refunds

## Instructions

| Instruction | Description |
|-------------|-------------|
| `create_campaign(goal, deadline)` | Initialize a campaign with a target amount and end time |
| `contribute(amount)` | Donate SOL to a campaign's vault |
| `withdraw` | Creator claims funds if campaign succeeded |
| `refund` | Donor reclaims contribution if campaign failed |

## Accounts

### Campaign PDA (`["campaign", creator]`)

| Field | Type | Description |
|-------|------|-------------|
| `creator` | `Pubkey` | Campaign owner |
| `goal` | `u64` | Target amount in lamports |
| `deadline` | `i64` | Unix timestamp for campaign end |
| `raised` | `u64` | Total lamports contributed |
| `claimed` | `bool` | Whether creator has withdrawn |
| `bump` | `u8` | PDA bump |

### Contribution PDA (`["contribution", campaign, donor]`)

| Field | Type | Description |
|-------|------|-------------|
| `donor` | `Pubkey` | Contributor's wallet |
| `amount` | `u64` | Lamports contributed |
| `bump` | `u8` | PDA bump |

### Vault PDA (`["vault", campaign]`)

A system-owned account that holds all donated SOL. No data — pure lamport storage.

## Error Codes

| Code | Condition |
|------|-----------|
| `DeadlineNotReached` | Action requires deadline to have passed |
| `GoalNotReached` | Withdraw requires goal to be met |
| `GoalAlreadyReached` | Refund not allowed if goal was met |
| `AlreadyClaimed` | Withdraw already executed |
| `CampaignExpired` | Contribution after deadline |
| `InvalidAmount` | Zero-amount contribution or goal |
| `InvalidDeadline` | Deadline in the past |

## Project Structure

```
crowdfunding/
├── programs/crowdfunding/src/
│   ├── lib.rs                    # Program entry point
│   ├── state.rs                  # Campaign & Contribution accounts
│   ├── error.rs                  # Custom error codes
│   ├── constants.rs              # Future constants
│   └── instructions/
│       ├── create_campaign.rs    # Initialize campaign
│       ├── contribute.rs         # Donate SOL
│       ├── withdraw.rs           # Creator claims funds
│       └── refund.rs             # Donor reclaims
├── Anchor.toml
├── Cargo.toml
└── package.json
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v1.0)
- [Node.js](https://nodejs.org/) (v18+)
- Yarn

### Build

```bash
cd crowdfunding/crowdfunding
anchor build
```

### Test

```bash
cargo test
```

### Deploy

```bash
anchor deploy --provider.cluster devnet
```

## Security Features

- PDA vault — no direct transfers to creator
- Double-withdrawal prevention via `claimed` flag
- Deadline enforcement — no early withdrawals
- Goal validation — refunds only if goal missed
- Checked arithmetic — no overflow
- Proper PDA seeds and bumps

## License

ISC
