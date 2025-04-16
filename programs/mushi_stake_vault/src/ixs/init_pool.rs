use {
    anchor_lang::prelude::*,
};

use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface,
};

use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};

use crate::{
    state::{MainState, VAULT_OWNER_SEED}, utils::{burn_tokens, mint_to_tokens_by_main_state}
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitPoolInput {
    pub stake_token_name: String,
    pub stake_token_symbol: String,
    pub stake_token_uri: String,
    pub mushi_program: Pubkey,
}

pub fn init_pool(
    ctx: Context<InitializeStakePool>,
    input: InitPoolInput,
) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    main_state.admin = ctx.accounts.admin.key();
    main_state.mushi_token_mint = ctx.accounts.mushi_token_mint.key();
    main_state.eclipse_token_mint = ctx.accounts.eclipse_token_mint.key();
    main_state.stake_token_mint = ctx.accounts.stake_token_mint.key();
    main_state.mushi_token_amount = 0;
    main_state.eclipse_token_amount = 0;
    main_state.mushi_program = input.mushi_program;

    let stake_token_vault = &mut ctx.accounts.stake_token_vault;
    let stake_token_program = &ctx.accounts.token_program;
    //mint stake tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.stake_token_mint.to_account_info(),
        main_state.to_account_info(),
        stake_token_vault.to_account_info(),
        stake_token_program.to_account_info(),
        1_000_000_000,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    //burn tokens
    burn_tokens(
        ctx.accounts.stake_token_vault.to_account_info(),
        ctx.accounts.stake_token_mint.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        stake_token_program.to_account_info(),
        1_000_000_000,
        Some(&[&[VAULT_OWNER_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]]),
    )?;

    // set token metadata
    let set_metadata_ix = CreateMetadataAccountV3 {
        metadata: ctx.accounts.stake_token_metadata_account.key(),
        mint: ctx.accounts.stake_token_mint.key(),
        mint_authority: main_state.key(),
        payer: ctx.accounts.admin.key(),
        rent: Some(ctx.accounts.rent.key()),
        system_program: ctx.accounts.system_program.key(),
        update_authority: (main_state.key(), true),
    }
    .instruction(CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: input.stake_token_name,
            symbol: input.stake_token_symbol,
            uri: input.stake_token_uri,
            creators: Some(vec![Creator {
                address: main_state.key(),
                share: 100,
                verified: true,
            }]),
            seller_fee_basis_points: 100,
            collection: None,
            uses: None,
        },
        is_mutable: false,
        collection_details: None,
    });
    invoke_signed(
        &set_metadata_ix,
        &[
            main_state.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.stake_token_mint.to_account_info(),
            ctx.accounts.stake_token_metadata_account.to_account_info(),
            ctx.accounts.mpl_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            MainState::PREFIX_SEED,
            &[*ctx.bumps.get("main_state").unwrap()],
        ]],
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeStakePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    // pool state account
    #[account(
        init,
        payer = admin,
        seeds = [MainState::PREFIX_SEED],
        bump,
        space =  8 + MainState::MAX_SIZE,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        mut,
    )]
    pub mushi_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        mint::token_program = token2022_program
    )]
    pub eclipse_token_mint: InterfaceAccount<'info, token_interface::Mint>,

    #[account(
        init,
        payer = admin,
        signer,
        mint::decimals = 9,
        mint::authority = main_state,
        mint::freeze_authority=main_state,
        mint::token_program = token_program,
    )]
    pub stake_token_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,

    ///CHECK:
    #[account(
        mut,
        seeds = [b"metadata", mpl_program.key.as_ref(), stake_token_mint.key().as_ref()],
        seeds::program = mpl_program,
        bump,
    )]
    pub stake_token_metadata_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [VAULT_OWNER_SEED],
        bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        associated_token::mint = mushi_token_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = token_program,
    )]
    pub mushi_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = eclipse_token_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = token2022_program,
    )]
    pub eclipse_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = stake_token_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = token_program,
    )]
    pub stake_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub token2022_program: Interface<'info, token_interface::TokenInterface>,
    ///CHECK:
    pub mpl_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
