use anchor_lang::prelude::*;

declare_id!("Bne2XHWW1HaMVHp6jXmCcmX3dVrtFMoYV5n2eyrvFw46");

#[program]
pub mod mushi_stake_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
