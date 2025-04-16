pub mod state;
pub mod ixs;
pub mod utils;
pub mod errors;

use anchor_lang::prelude::*;
use ixs::{init_pool::*, stake::*, unstake::*, update_pool::*};
pub use ixs::stake::StakeInput;
pub use ixs::unstake::UnstakeInput;
pub use ixs::update_pool::UpdatePoolInput;
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
    
    pub fn update_pool(ctx: Context<UpdatePool>, input: UpdatePoolInput) -> Result<()> {
        ixs::update_pool::handler(ctx, input)
    }
}
