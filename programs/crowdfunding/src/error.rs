use anchor_lang::prelude::*;

#[error_code]
pub enum CrowdfundingError {
    /// Action requires deadline to have passed
    #[msg("Deadline has not been reached yet")]
    DeadlineNotReached,

    /// Withdraw requires goal to be met
    #[msg("Funding goal has not been reached")]
    GoalNotReached,

    /// Refund not allowed if goal was met
    #[msg("Funding goal was already reached; refunds are not available")]
    GoalAlreadyReached,

    /// Withdraw already executed
    #[msg("Funds have already been claimed by the creator")]
    AlreadyClaimed,

    /// Contribution after deadline
    #[msg("Campaign has expired; contributions are no longer accepted")]
    CampaignExpired,

    /// Zero-amount contribution or goal
    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    /// Deadline in the past
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
}