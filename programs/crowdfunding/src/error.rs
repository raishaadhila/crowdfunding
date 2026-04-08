use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Deadline has not been reached yet")]
    DeadlineNotReached,
    #[msg("Goal has not been reached")]
    GoalNotReached,
    #[msg("Goal was reached, refunds not allowed")]
    GoalAlreadyReached,
    #[msg("Funds already claimed")]
    AlreadyClaimed,
    #[msg("Campaign has expired")]
    CampaignExpired,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
}
