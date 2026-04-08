use anchor_lang::prelude::*;

/// On-chain state for a crowdfunding campaign.
///
/// PDA seeds: ["campaign", creator]
/// One campaign per creator wallet.
#[account]
#[derive(InitSpace)]
pub struct Campaign {
    /// The wallet that created this campaign and will receive funds on success.
    pub creator: Pubkey,

    /// Fundraising target in lamports. Must be > 0.
    pub goal: u64,

    /// Unix timestamp after which withdrawal or refund becomes available.
    pub deadline: i64,

    /// Running total of lamports contributed to the vault.
    pub raised: u64,

    /// Set to `true` after the creator has successfully withdrawn funds.
    /// Prevents double-withdrawal.
    pub claimed: bool,

    /// Canonical bump for this PDA; stored to avoid recomputation.
    pub bump: u8,

    /// Canonical bump for the vault PDA associated with this campaign.
    pub vault_bump: u8,
}

/// Records a single donor's total contribution to a campaign.
///
/// PDA seeds: ["contribution", campaign, donor]
/// Initialised on first contribution; updated on subsequent ones.
/// Closed (rent reclaimed) when a refund is issued.
#[account]
#[derive(InitSpace)]
pub struct Contribution {
    /// The donor's wallet address.
    pub donor: Pubkey,

    /// Total lamports this donor has sent to the vault.
    pub amount: u64,

    /// Canonical bump for this PDA.
    pub bump: u8,
}