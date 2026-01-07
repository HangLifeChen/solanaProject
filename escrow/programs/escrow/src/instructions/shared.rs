use anchor_lang::prelude::*;
use anchor_spl::{token_2022::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};


pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info,TokenAccount>,
    to: &InterfaceAccount<'info,TokenAccount>,
    authority: &Signer<'info>,
    mint: &InterfaceAccount<'info,Mint>,
    amount: &u64,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let transfer_account = TransferChecked {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
        mint: mint.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        token_program.to_account_info(),
        transfer_account,
    );
    transfer_checked(cpi_context, *amount, 6)
}