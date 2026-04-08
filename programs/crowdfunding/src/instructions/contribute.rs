use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::{CAMPAIGN_SEED, CONTRIBUTION_SEED, VAULT_SEED},
    error::CrowdfundingError,
    state::{Campaign, Contribution},
};

/// Accounts required to contribute SOL to a campaign.
#[derive(Accounts)]
pub struct Contribute<'info> {
    /// The donor — signs and pays.
    #[account(mut)]
    pub donor: Signer<'info>,

    /// The campaign being funded.
    #[account(
        mut,
        seeds = [CAMPAIGN_SEED, campaign.creator.as_ref()],
        bump = campaign.bump,
    )]
    pub campaign: Account<'info, Campaign>,

    /// Vault that holds donated lamports.
    ///
    /// CHECK: PDA-controlled lamport vault. Safety enforced by seeds
    /// constraint and the fact that we only transfer *into* it here.
    #[account(
        mut,
        seeds = [VAULT_SEED, campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// Per-(campaign, donor) contribution record.
    /// `init_if_needed` allows repeat contributions without a separate
    /// instruction. Re-initialisation is safe here because we always
    /// *add* to `amount` — we never reset it — and the donor's key is
    /// verified via the seeds constraint.
    #[account(
        init_if_needed,
        payer = donor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [CONTRIBUTION_SEED, campaign.key().as_ref(), donor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    pub system_program: Program<'info, System>,
}

/// Transfers `amount` lamports from the donor to the vault and updates
/// the on-chain bookkeeping.
///
/// # Arguments
/// * `amount` — lamports to donate (must be > 0)
pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    require!(amount > 0, CrowdfundingError::InvalidAmount);

    let now = Clock::get()?.unix_timestamp;
    require!(
        now < ctx.accounts.campaign.deadline,
        CrowdfundingError::CampaignExpired,
    );

    // Transfer SOL: donor → vault (system-program CPI)
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update campaign total — checked arithmetic prevents overflow.
    let campaign = &mut ctx.accounts.campaign;
    campaign.raised = campaign.raised.checked_add(amount).unwrap();

    // Update (or initialise) the contribution record.
    let contribution = &mut ctx.accounts.contribution;
    if contribution.donor == Pubkey::default() {
        // First contribution: initialise the record.
        contribution.donor = ctx.accounts.donor.key();
        contribution.bump = ctx.bumps.contribution;
    }
    contribution.amount = contribution.amount.checked_add(amount).unwrap();

    msg!(
        "Contribution received | donor={} amount={} total_raised={}",
        contribution.donor,
        amount,
        campaign.raised,
    );

    Ok(())
}