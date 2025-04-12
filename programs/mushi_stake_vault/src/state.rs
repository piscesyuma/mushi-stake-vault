use anchor_lang::prelude::*;
pub const VAULT_SEED: &'static [u8] = b"vault";
pub const VAULT_AUTH_SEED: &'static [u8] = b"vault_authority";
pub const STAKE_ENTRY_SEED: &'static [u8] = b"stake_entry";

#[account]
pub struct MainState {
    pub admin: Pubkey,
    pub mushi_token_amount: u64,
    pub eclipse_token_amount: u64,
    pub mushi_token_mint: Pubkey,
    pub eclipse_token_mint: Pubkey,
    pub staking_token_mint: Pubkey,
}

impl MainState {
    pub const PREFIX_SEED: &'static [u8] = b"main_state";
    pub const MAX_SIZE: usize = 8 + std::mem::size_of::<MainState>();
}

#[account]
pub struct StakeEntry {
    pub user: Pubkey,
    pub user_stake_token_account: Pubkey,
    pub bump: u8,
    pub mushi_token_balance: u64,
    pub eclipse_token_balance: u64,
    pub last_staked: i64,
}