use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::{state::{Campaign, Contribution}, error::ErrorCode};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(
        seeds = [b"campaign", campaign.creator.as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(
        mut,
        close = donor,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump = contribution.bump,
        has_one = donor
    )]
    pub contribution: Account<'info, Contribution>,
    /// CHECK: PDA vault — lamport-only account
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub donor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Refund>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let campaign = &mut ctx.accounts.campaign;

    require!(now >= campaign.deadline, ErrorCode::DeadlineNotReached);
    require!(campaign.raised < campaign.goal, ErrorCode::GoalAlreadyReached);

    let amount = ctx.accounts.contribution.amount;

    campaign.raised = campaign.raised.checked_sub(amount).ok_or(ErrorCode::InvalidAmount)?;

    system_program::transfer(
        CpiContext::new_with_signer(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.donor.to_account_info(),
            },
            &[&[b"vault", campaign.key().as_ref(), &[campaign.vault_bump]]],
        ),
        amount,
    )?;

    Ok(())
}
