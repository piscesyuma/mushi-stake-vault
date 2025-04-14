use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};
use crate::{
    errors::MushiStakeVaultError, state::{MainState, VAULT_OWNER_SEED}, utils::{burn_tokens, transfer_token_2022, transfer_tokens, TransferToken2022Input, TransferTokenInput}
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct UnstakeInput {
    pub amount: u64,
}

pub fn handler(ctx: Context<Unstake>, input: UnstakeInput) -> Result<()> {
    let eclipse_token_amount = input.amount;
    let mushi_token_amount = input.amount;
    let stake_token_amount = input.amount;

    require!(mushi_token_amount <= ctx.accounts.mushi_token_vault.amount, MushiStakeVaultError::InsufficientMushiTokenAmount);
    require!(eclipse_token_amount <= ctx.accounts.eclipse_token_vault.amount, MushiStakeVaultError::InsufficientEclipseTokenAmount);
    
    let bump = *ctx.bumps.get("token_vault_owner").unwrap();
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_OWNER_SEED, &[bump]]];

    transfer_tokens(
        TransferTokenInput {
            from: ctx.accounts.mushi_token_vault.to_account_info(),
            to: ctx.accounts.user_mushi_token_ata.to_account_info(),
            authority: ctx.accounts.token_vault_owner.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: mushi_token_amount,
        },
        Some(signer_seeds),
    )?;

    transfer_token_2022(
        TransferToken2022Input {
            from: ctx.accounts.user_eclipse_token_ata.to_account_info(),
            to: ctx.accounts.eclipse_token_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
            mint: ctx.accounts.eclipse_token_mint.to_account_info(),
            token_program: ctx.accounts.token2022_program.to_account_info(),
            amount: eclipse_token_amount,
            decimals: ctx.accounts.eclipse_token_mint.decimals,
        },
        Some(signer_seeds),
    )?;

    burn_tokens(
        ctx.accounts.user_stake_token_ata.to_account_info(),
        ctx.accounts.stake_token_mint.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        stake_token_amount,
        None,
    )?;

    
    let main_state = &mut ctx.accounts.main_state;
    main_state.mushi_token_amount -= mushi_token_amount;
    main_state.eclipse_token_amount -= eclipse_token_amount;
    main_state.staking_token_total_supply -= stake_token_amount;
    Ok(())
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(    
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        mut,
        associated_token::mint = mushi_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_mushi_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        token::mint = eclipse_token_mint,
        token::authority = user,
        token::token_program = token2022_program,
    )]
    pub user_eclipse_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stake_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_stake_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mushi_token_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = token_program,
    )]
    pub mushi_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = main_state.mushi_token_mint,
    )]
    pub mushi_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        token::mint = eclipse_token_mint,
        token::authority = token_vault_owner,
        token::token_program = token2022_program,
    )]
    pub eclipse_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token2022_program,
        address = main_state.eclipse_token_mint,
    )]
    pub eclipse_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = main_state.stake_token_mint,
    )]
    pub stake_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        seeds = [VAULT_OWNER_SEED],
        bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub token2022_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
}
