use anchor_lang::prelude::*;

use crate::{
    constants::{CAMPAIGN_SEED, VAULT_SEED},
    error::CrowdfundingError,
    state::Campaign,
};

/// Accounts required for the creator to withdraw funds after a
/// successful campaign.
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The campaign creator — must match `campaign.creator`.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The campaign account — validates creator identity & state.
    #[account(
        mut,
        seeds = [CAMPAIGN_SEED, creator.key().as_ref()],
        bump = campaign.bump,
        has_one = creator,
    )]
    pub campaign: Account<'info, Campaign>,

    /// Vault holding the donated lamports.
    ///
    /// CHECK: PDA-controlled lamport vault. We verify the seeds here and
    /// use `invoke_signed` with the stored bump to transfer out — no
    /// arbitrary data is read or written.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Transfers all vault lamports to the creator.
///
/// Conditions that must hold:
/// * deadline has passed
/// * raised ≥ goal
/// * funds not yet claimed
pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    require!(
        now >= ctx.accounts.campaign.deadline,
        CrowdfundingError::DeadlineNotReached,
    );
    require!(
        ctx.accounts.campaign.raised >= ctx.accounts.campaign.goal,
        CrowdfundingError::GoalNotReached,
    );
    require!(
        !ctx.accounts.campaign.claimed,
        CrowdfundingError::AlreadyClaimed,
    );

    // Mark as claimed *before* the transfer (checks-effects-interactions).
    ctx.accounts.campaign.claimed = true;

    // Drain vault → creator via raw lamport manipulation.
    // The vault has no data so we cannot use Anchor's `close` helper —
    // we move lamports manually using `invoke_signed`.
    let vault_lamports = ctx.accounts.vault.lamports();

    // Transfer via system program CPI signed by the vault PDA.
    let campaign_key = ctx.accounts.campaign.key();
    let vault_seeds: &[&[u8]] = &[
        VAULT_SEED,
        campaign_key.as_ref(),
        &[ctx.accounts.campaign.vault_bump],
    ];

    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.vault.key,
            ctx.accounts.creator.key,
            vault_lamports,
        ),
        &[
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[vault_seeds],
    )?;

    msg!(
        "Withdraw successful | creator={} amount={}",
        ctx.accounts.creator.key(),
        vault_lamports,
    );

    Ok(())
}