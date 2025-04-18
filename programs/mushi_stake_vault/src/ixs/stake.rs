use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};
use crate::{
    errors::MushiStakeVaultError, state::{MainState, VAULT_OWNER_SEED}, utils::{mint_to_tokens_by_main_state, TransferTokenInput, transfer_tokens, TransferToken2022Input, transfer_token_2022}
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StakeInput {
    pub amount: u64,
}

pub fn handler(ctx: Context<Stake>, input: StakeInput) -> Result<()> {
    // Verify that this function is being called by the authorized program
    let instructions_sysvar = ctx.accounts.instruction_sysvar.as_ref();
    
    // Get the current instruction index
    let current_index = anchor_lang::solana_program::sysvar::instructions::load_current_index_checked(instructions_sysvar)?;
    
    // Get the current instruction and check who called us
    let current_ix = anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked(
        current_index as usize, 
        instructions_sysvar
    )?;
    
    // Check if the current instruction's program ID matches the mushi_program
    if current_ix.program_id != ctx.accounts.main_state.mushi_program {
        // Check if we were called by the mushi_program via CPI
        if current_index == 0 {
            // If we're the first instruction, there's no caller, so unauthorized
            return Err(MushiStakeVaultError::UnauthorizedProgramCall.into());
        }
        
        // Check the previous instruction to see if it's the authorized program
        let prev_ix = anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked(
            current_index as usize - 1, 
            instructions_sysvar
        )?;
        
        if prev_ix.program_id != ctx.accounts.main_state.mushi_program {
            return Err(MushiStakeVaultError::UnauthorizedProgramCall.into());
        }
    }

    let eclipse_token_amount = input.amount;
    let mushi_token_amount = input.amount;
    let stake_token_amount = input.amount;

    require!(mushi_token_amount <= ctx.accounts.user_mushi_token_ata.amount, MushiStakeVaultError::InsufficientMushiTokenAmount);

    transfer_tokens(
        TransferTokenInput {
            from: ctx.accounts.user_mushi_token_ata.to_account_info(),
            to: ctx.accounts.mushi_token_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: mushi_token_amount,
        },
        None,
    )?;

    // Transfer eclipse token from user to vault
    // transfer_token_2022(
    //     TransferToken2022Input {
    //         from: ctx.accounts.user_eclipse_token_ata.to_account_info(),
    //         to: ctx.accounts.eclipse_token_vault.to_account_info(),
    //         authority: ctx.accounts.user.to_account_info(),
    //         mint: ctx.accounts.eclipse_token_mint.to_account_info(),
    //         token_program: ctx.accounts.token2022_program.to_account_info(),
    //         amount: eclipse_token_amount,
    //         decimals: ctx.accounts.eclipse_token_mint.decimals,
    //     },
    //     None,
    // )?;

    mint_to_tokens_by_main_state(
        ctx.accounts.stake_token_mint.to_account_info(),
        ctx.accounts.main_state.to_account_info(),
        ctx.accounts.user_stake_token_ata.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        stake_token_amount,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    
    let main_state = &mut ctx.accounts.main_state;
    main_state.mushi_token_amount += mushi_token_amount;
    main_state.eclipse_token_amount += eclipse_token_amount;
    main_state.staking_token_total_supply += stake_token_amount;
    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(    
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    /// CHECK: This is the Solana instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = mushi_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_mushi_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
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
