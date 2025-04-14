pub mod state;
pub mod ixs;
pub mod utils;
pub mod errors;

use {anchor_lang::prelude::*, ixs::* };

declare_id!("Bne2XHWW1HaMVHp6jXmCcmX3dVrtFMoYV5n2eyrvFw46");

#[program]
pub mod mushi_stake_vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeStakePool>, input: InitPoolInput) -> Result<()> {
        ixs::init_pool(ctx, input)
    }

    pub fn stake(ctx: Context<Stake>, input: StakeInput) -> Result<()> {
        ixs::stake::handler(ctx, input)
    }

    pub fn unstake(ctx: Context<Unstake>, input: UnstakeInput) -> Result<()> {
        ixs::unstake::handler(ctx, input)
    }
    
    
}
