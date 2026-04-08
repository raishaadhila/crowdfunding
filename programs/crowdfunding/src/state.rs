use anchor_lang::prelude::*;

pub const SEED: &str = "anchor";

#[account]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub deadline: i64,
    pub raised: u64,
    pub claimed: bool,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Campaign {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 1;
}

#[account]
pub struct Contribution {
    pub donor: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl Contribution {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}
