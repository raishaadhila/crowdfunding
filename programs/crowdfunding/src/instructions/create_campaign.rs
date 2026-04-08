use anchor_lang::prelude::*;
use crate::{state::Campaign, error::ErrorCode};

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::LEN,
        seeds = [b"campaign", creator.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateCampaign>, goal: u64, deadline: i64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(goal > 0, ErrorCode::InvalidAmount);
    require!(deadline > now, ErrorCode::InvalidDeadline);

    let campaign = &mut ctx.accounts.campaign;
    campaign.creator = ctx.accounts.creator.key();
    campaign.goal = goal;
    campaign.deadline = deadline;
    campaign.raised = 0;
    campaign.claimed = false;
    campaign.bump = ctx.bumps.campaign;
    Ok(())
}
