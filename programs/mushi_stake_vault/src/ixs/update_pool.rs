use anchor_lang::prelude::*;
use crate::{state::MainState, errors::MushiStakeVaultError};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct UpdatePoolInput {
    pub mushi_program: Pubkey,
}

pub fn handler(ctx: Context<UpdatePool>, input: UpdatePoolInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    main_state.mushi_program = input.mushi_program;
    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    #[account(
        address = main_state.admin @ MushiStakeVaultError::UnauthorizedAdminAction,
        mut
    )]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    pub system_program: Program<'info, System>,
} 