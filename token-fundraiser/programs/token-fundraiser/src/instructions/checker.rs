use anchor_lang::accounts::signer;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{transfer_checked, TransferChecked};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::Fundraiser;
use crate::FundraiserError;

#[derive(Accounts)]
pub struct CheckContributions<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_to_raise: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds=[b"fundraiser", maker.key().as_ref()],
        bump=fundraiser.bump,
        // close=maker
    )]
    pub fundraiser: Account<'info, Fundraiser>,
    #[account(
       mut,
       associated_token::mint=mint_to_raise,
       associated_token::authority=fundraiser,
       associated_token::token_program=token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
     #[account(
        init_if_needed,
        payer=maker,
        associated_token::mint=mint_to_raise,
        associated_token::authority=maker,
        associated_token::token_program=token_program,
    )]
    pub maker_ata:InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info>CheckContributions<'info> {
    pub fn check_contributions(&self) -> Result<()> {
        require!(self.vault.amount<=self.fundraiser.amount_to_raise,
             FundraiserError::TargetNotMet);

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts=TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            mint: self.mint_to_raise.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };
        let binding = self.maker.to_account_info().key();
        let signer_seeds:&[&[&[u8]];1] = &[
            &[b"fundraiser", binding.as_ref(), &[self.fundraiser.bump]
            ]];
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_to_raise.decimals)
    }
}