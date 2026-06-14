# Solana Crowdfunding Platform — PRD

## Overview
A crowdfunding smart contract on Solana. Think Kickstarter, but on-chain. Creators launch campaigns with a goal and deadline; donors contribute SOL into a PDA vault; funds are released to the creator only if the goal is met after the deadline, otherwise donors can reclaim their contributions.

---

## Problem Statement
Creators currently have no trustless way to:
- Hold donations in escrow until conditions are met
- Guarantee refunds if a goal isn't reached
- Prove to donors that funds are locked and rule-enforced

---

## Actors
| Actor | Role |
|-------|------|
| Creator | Creates a campaign, sets goal + deadline, withdraws if successful |
| Donor | Contributes SOL, gets refund if campaign fails |

---

## Accounts

### `Campaign` (PDA: `["campaign", creator]`)
| Field | Type | Description |
|-------|------|-------------|
| `creator` | `Pubkey` | Campaign owner |
| `goal` | `u64` | Target amount in lamports |
| `deadline` | `i64` | Unix timestamp for campaign end |
| `raised` | `u64` | Total lamports contributed |
| `claimed` | `bool` | Whether creator has withdrawn |
| `bump` | `u8` | PDA bump |

### `Contribution` (PDA: `["contribution", campaign, donor]`)
| Field | Type | Description |
|-------|------|-------------|
| `donor` | `Pubkey` | Contributor's wallet |
| `amount` | `u64` | Lamports contributed |
| `bump` | `u8` | PDA bump |

### Vault (PDA: `["vault", campaign]`)
A system-owned account that holds all donated SOL. No data — pure lamport storage.

---

## Instructions

### `create_campaign(goal: u64, deadline: i64)`
- Initializes `Campaign` PDA
- Validates: `goal > 0`, `deadline > now`

### `contribute(amount: u64)`
- Transfers `amount` lamports from donor → vault
- Initializes or updates `Contribution` PDA
- Increments `campaign.raised`
- Validates: campaign not expired, `amount > 0`

### `withdraw`
- Transfers all vault lamports → creator
- Sets `campaign.claimed = true`
- Validates: `now >= deadline`, `raised >= goal`, `!claimed`

### `refund`
- Transfers donor's contribution from vault → donor
- Closes `Contribution` PDA
- Validates: `now >= deadline`, `raised < goal`

---

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

---

## Success Criteria
- ✅ Accept campaign creation with goal and deadline
- ✅ Accept contributions and track total raised
- ✅ Allow withdrawal only if goal reached after deadline
- ✅ Allow refunds only if goal NOT reached after deadline
- ✅ Prevent double withdrawals (`claimed` flag)
- ✅ Use PDA vault (no direct transfers to creator)

---

## Testing Checklist
1. Create campaign: `goal=1000 SOL`, `deadline=tomorrow` → success
2. Contribute 600 SOL → success, `raised=600`
3. Contribute 500 SOL → success, `raised=1100`
4. Withdraw before deadline → fail (`DeadlineNotReached`)
5. Withdraw after deadline, goal met → success
6. Withdraw again → fail (`AlreadyClaimed`)
7. Contribute after deadline → fail (`CampaignExpired`)
8. Refund when goal met → fail (`GoalAlreadyReached`)
9. Refund when goal not met, after deadline → success

---

## Tech Stack
- Anchor 1.0 (Solana program framework)
- LiteSVM (unit testing)
- Rust 2021 edition
