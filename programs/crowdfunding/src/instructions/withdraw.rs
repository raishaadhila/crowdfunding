use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::{state::Campaign, error::ErrorCode};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"campaign", creator.key().as_ref()],
        bump = campaign.bump,
        has_one = creator
    )]
    pub campaign: Account<'info, Campaign>,
    /// CHECK: PDA vault — lamport-only account
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Withdraw>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let campaign = &mut ctx.accounts.campaign;

    require!(now >= campaign.deadline, ErrorCode::DeadlineNotReached);
    require!(campaign.raised >= campaign.goal, ErrorCode::GoalNotReached);
    require!(!campaign.claimed, ErrorCode::AlreadyClaimed);

    campaign.claimed = true;

    let amount = campaign.raised;

    system_program::transfer(
        CpiContext::new_with_signer(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
            &[&[b"vault", campaign.key().as_ref(), &[campaign.vault_bump]]],
        ),
        amount,
    )?;

    Ok(())
}
