use anchor_lang::prelude::*;

use crate::{
    constants::{CAMPAIGN_SEED, VAULT_SEED},
    error::CrowdfundingError,
    state::Campaign,
};

/// Accounts required to create a new campaign.
#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    /// The wallet creating the campaign. Pays for account rent.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Campaign PDA — one per creator.
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE,
        seeds = [CAMPAIGN_SEED, creator.key().as_ref()],
        bump,
    )]
    pub campaign: Account<'info, Campaign>,

    /// Vault PDA — pure lamport storage, no data.
    /// We derive it here only to capture the bump; funds are not
    /// transferred during campaign creation.
    ///
    /// CHECK: This is a PDA used as a lamport vault. It has no data and
    /// is never written as an Anchor account — safety is enforced by the
    /// seeds constraint and the program logic in contribute/withdraw/refund.
    #[account(
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Initialises a new crowdfunding campaign.
///
/// # Arguments
/// * `goal`     — fundraising target in lamports (must be > 0)
/// * `deadline` — Unix timestamp when the campaign ends (must be in the future)
pub fn create_campaign(
    ctx: Context<CreateCampaign>,
    goal: u64,
    deadline: i64,
) -> Result<()> {
    require!(goal > 0, CrowdfundingError::InvalidAmount);

    let now = Clock::get()?.unix_timestamp;
    require!(deadline > now, CrowdfundingError::InvalidDeadline);

    let campaign = &mut ctx.accounts.campaign;
    campaign.creator = ctx.accounts.creator.key();
    campaign.goal = goal;
    campaign.deadline = deadline;
    campaign.raised = 0;
    campaign.claimed = false;
    campaign.bump = ctx.bumps.campaign;
    campaign.vault_bump = ctx.bumps.vault;

    msg!(
        "Campaign created | creator={} goal={} deadline={}",
        campaign.creator,
        campaign.goal,
        campaign.deadline,
    );

    Ok(())
}