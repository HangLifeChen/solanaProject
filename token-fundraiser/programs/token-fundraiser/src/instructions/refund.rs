use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{error::FundraiserError, state::{Contributor, Fundraiser}, SECONDS_TO_DAYS};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub mint_to_raise: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        has_one=maker,
        has_one=mint_to_raise,
        seeds=[b"fundraiser", maker.key().as_ref()],
        bump=fundraiser.bump,
    )]
    fundraiser: Account<'info, Fundraiser>,
    #[account(
        mut,
        associated_token::mint=mint_to_raise,
        associated_token::authority=fundraiser,
        associated_token::token_program=token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds=[b"contributor",fundraiser.key().as_ref(),contributor.key().as_ref()],
        bump,
        close=contributor
    )]
    pub contributor_account: Account<'info, Contributor>,
    #[account(
        mut,
        associated_token::mint=mint_to_raise,
        associated_token::authority=contributor,
        associated_token::token_program=token_program,
    )]
    pub contributor_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info>Refund<'info>{
    pub fn refund(&mut self) -> Result<()> {
        let current_time=Clock::get()?.unix_timestamp;
        require!(
            self.fundraiser.duration>=((current_time-self.fundraiser.start_time)/SECONDS_TO_DAYS)as u16,
            FundraiserError::FundraiserNotEnded
        );

        require!(
            self.vault.amount < self.fundraiser.amount_to_raise,
            FundraiserError::TargetNotMet
        );

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts=TransferChecked {
            from: self.vault.to_account_info(),
            to: self.contributor_ata.to_account_info(),
            mint: self.mint_to_raise.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };

         let signer_seeds: &[&[&[u8]]; 1] = &[&[
            b"fundraiser",
            self.maker.to_account_info().key.as_ref(),
            &[self.fundraiser.bump],
        ]];

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
        transfer_checked(cpi_ctx, self.contributor_account.amount, self.mint_to_raise.decimals)?;
        self.fundraiser.current_amount = self.fundraiser.current_amount.checked_sub(self.contributor_account.amount).unwrap();

        Ok(())
    }
}