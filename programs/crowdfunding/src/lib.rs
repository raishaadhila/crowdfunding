use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

// Replace with your actual program ID after `anchor keys list`
declare_id!("Fs4wNeXAJKypKodn4kbDs7nmdJeFB6emVY2Ntj3FLqzt");

#[program]
pub mod crowdfunding {
    use super::*;
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        goal: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::create_campaign::create_campaign(ctx, goal, deadline)
    }

    /// Contribute SOL to a campaign's vault.
    ///
    /// # Arguments
    /// * `amount` — lamports to donate (> 0)
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        instructions::contribute::contribute(ctx, amount)
    }

    /// Withdraw all vault funds to the creator (campaign succeeded).
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw::withdraw(ctx)
    }

    /// Refund a donor's contribution (campaign failed).
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::refund(ctx)
    }
}