pub mod state;
pub mod ixs;
pub mod utils;
use {anchor_lang::prelude::*, ixs::*, utils::*};

declare_id!("Bne2XHWW1HaMVHp6jXmCcmX3dVrtFMoYV5n2eyrvFw46");

#[program]
pub mod mushi_stake_vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeStakePool>, input: InitPoolInput) -> Result<()> {
        ixs::init_pool(ctx, input)
    }
}
