use crate::state::MainState;

use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};
use anchor_spl::{
    token::{self, Burn, MintTo, Token, TokenAccount, Transfer},
    token_2022::{self, transfer_checked, TransferChecked},
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

pub fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let token_transfer_accounts = Transfer {
        from,
        to,
        authority,
    };
    if let Some(signer_seeds) = signer_seeds {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone(),
                token_transfer_accounts,
                signer_seeds,
            ),
            amount,
        )?;
    } else {
        token::transfer(
            CpiContext::new(token_program, token_transfer_accounts),
            amount,
        )?;
    }
    Ok(())
}
