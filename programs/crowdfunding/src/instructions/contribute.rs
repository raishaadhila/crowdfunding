use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::{state::{Campaign, Contribution}, error::ErrorCode};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(
        mut,
        seeds = [b"campaign", campaign.creator.as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(
        init_if_needed,
        payer = donor,
        space = Contribution::LEN,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,
    /// CHECK: PDA vault — holds lamports only, no data
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

pub fn handler(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let campaign = &mut ctx.accounts.campaign;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(now < campaign.deadline, ErrorCode::CampaignExpired);

    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    campaign.raised = campaign.raised.checked_add(amount).ok_or(ErrorCode::InvalidAmount)?;

    let contribution = &mut ctx.accounts.contribution;
    contribution.donor = ctx.accounts.donor.key();
    contribution.amount = contribution.amount.checked_add(amount).ok_or(ErrorCode::InvalidAmount)?;
    contribution.bump = ctx.bumps.contribution;

    Ok(())
}
