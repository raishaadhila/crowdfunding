/// Seed used to derive the Campaign PDA
pub const CAMPAIGN_SEED: &[u8] = b"campaign";

/// Seed used to derive the Vault PDA (pure lamport storage, no data)
pub const VAULT_SEED: &[u8] = b"vault";

/// Seed used to derive a Contribution PDA per (campaign, donor) pair
pub const CONTRIBUTION_SEED: &[u8] = b"contribution";