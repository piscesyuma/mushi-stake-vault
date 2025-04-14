use crate::state::MainState;

use anchor_lang::{
    prelude::*,
};
use anchor_spl::{
    token::{self, Burn, MintTo, Transfer},
    token_2022::{self, TransferChecked},
};

pub fn mint_to_tokens_by_main_state<'info>(
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    receiver_ata: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let accounts = MintTo {
        authority,
        mint,
        to: receiver_ata,
    };
    token::mint_to(
        CpiContext::new_with_signer(
            token_program,
            accounts,
            &[&[MainState::PREFIX_SEED, &[bump]]],
        ),
        amount,
    )
}

pub fn burn_tokens<'info>(
    token_account: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let accounts = Burn {
        authority,
        from: token_account,
        mint,
    };
    if let Some(signer_seeds) = signer_seeds {
        token::burn(
            CpiContext::new_with_signer(token_program, accounts, signer_seeds),
            amount,
        )
    } else {
        token::burn(CpiContext::new(token_program, accounts), amount)
    }
}

pub struct TransferTokenInput<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub amount: u64,
}

pub fn transfer_tokens(
    input: TransferTokenInput<'_>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let token_transfer_accounts = Transfer {
        from: input.from,
        to: input.to,
        authority: input.authority,
    };
    if let Some(signer_seeds) = signer_seeds {
        token::transfer(
            CpiContext::new_with_signer(
                input.token_program.clone(),
                token_transfer_accounts,
                signer_seeds,
            ),
            input.amount,
        )?;
    } else {
        token::transfer(
            CpiContext::new(input.token_program.clone(), token_transfer_accounts),
            input.amount,
        )?;
    }
    Ok(())
}

pub struct TransferToken2022Input<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub amount: u64,
    pub decimals: u8,
}

pub fn transfer_token_2022(
    input: TransferToken2022Input<'_>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let token_transfer_accounts = TransferChecked {
        from: input.from,
        to: input.to,
        authority: input.authority,
        mint: input.mint,
    };
    if let Some(signer_seeds) = signer_seeds {
        token_2022::transfer_checked(
            CpiContext::new_with_signer(input.token_program.clone(), token_transfer_accounts, signer_seeds),
            input.amount,
            input.decimals,
        )?;
    } else {
        token_2022::transfer_checked(
            CpiContext::new(input.token_program.clone(), token_transfer_accounts),
            input.amount,
            input.decimals,
        )?;
    }
    Ok(())
}