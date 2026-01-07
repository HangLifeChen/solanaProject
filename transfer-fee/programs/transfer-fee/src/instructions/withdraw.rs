use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::{withdraw_withheld_tokens_from_mint, Mint, TokenAccount, WithdrawWithheldTokensFromMint}};

#[derive(Accounts)]
pub struct Withdraw<'info>{
    pub authority: Signer<'info>,
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
}


pub fn process_withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let token_account = &ctx.accounts.token_account;
    let mint_account = &ctx.accounts.mint_account;
    let token_program = &ctx.accounts.token_program;

    // Logic to withdraw tokens from the token account
    // This could involve transferring tokens back to the authority or another account

    msg!("Withdrawing tokens from {} to {}", token_account.key(), authority.key());

    // Example logic: transfer all tokens back to the authority
    let cpi_accounts = WithdrawWithheldTokensFromMint {
        token_program_id: token_program.to_account_info(),
        mint: mint_account.to_account_info(),
        destination: token_account.to_account_info(),
        authority: authority.to_account_info(),
    };

    withdraw_withheld_tokens_from_mint(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
    )?;

    Ok(())
}