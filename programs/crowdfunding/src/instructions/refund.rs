use anchor_lang::prelude::*;

use crate::{
    constants::{CAMPAIGN_SEED, CONTRIBUTION_SEED, VAULT_SEED},
    error::CrowdfundingError,
    state::{Campaign, Contribution},
};

/// Accounts required for a donor to reclaim their contribution after a
/// failed campaign.
#[derive(Accounts)]
pub struct Refund<'info> {
    /// The donor reclaiming their funds.
    #[account(mut)]
    pub donor: Signer<'info>,

    /// The campaign that did not meet its goal.
    #[account(
        mut,
        seeds = [CAMPAIGN_SEED, campaign.creator.as_ref()],
        bump = campaign.bump,
    )]
    pub campaign: Account<'info, Campaign>,

    /// Vault from which the refund is paid.
    ///
    /// CHECK: PDA-controlled lamport vault. Seeds verified; we only
    /// transfer *out* an amount we previously transferred *in*.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// The donor's contribution record — closed after refund so rent is
    /// returned to the donor.
    #[account(
        mut,
        seeds = [CONTRIBUTION_SEED, campaign.key().as_ref(), donor.key().as_ref()],
        bump = contribution.bump,
        has_one = donor,
        close = donor,
    )]
    pub contribution: Account<'info, Contribution>,

    pub system_program: Program<'info, System>,
}

/// Returns the donor's contributed lamports from the vault.
///
/// Conditions that must hold:
/// * deadline has passed
/// * raised < goal (campaign failed)
pub fn refund(ctx: Context<Refund>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    require!(
        now >= ctx.accounts.campaign.deadline,
        CrowdfundingError::DeadlineNotReached,
    );
    require!(
        ctx.accounts.campaign.raised < ctx.accounts.campaign.goal,
        CrowdfundingError::GoalAlreadyReached,
    );

    let refund_amount = ctx.accounts.contribution.amount;

    // Decrement campaign total so the accounting stays consistent.
    ctx.accounts.campaign.raised = ctx
        .accounts
        .campaign
        .raised
        .checked_sub(refund_amount)
        .unwrap();

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
            ctx.accounts.donor.key,
            refund_amount,
        ),
        &[
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.donor.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[vault_seeds],
    )?;

    msg!(
        "Refund issued | donor={} amount={}",
        ctx.accounts.donor.key(),
        refund_amount,
    );

    Ok(())
}