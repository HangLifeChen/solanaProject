use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token_2022::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};
use crate::{constants::ANCHOR_DISCRIMINATOR, error::FundraiserError, state::{Contributor, Fundraiser}, MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,
    pub mint_to_raise: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        has_one=mint_to_raise,
        seeds=[b"fundraiser", fundraiser.maker.key().as_ref()],
        bump=fundraiser.bump,
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
        payer=contributor,
        seeds=[b"contributor", fundraiser.key().as_ref(), contributor.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Contributor::INIT_SPACE,
    )]
    pub contributor_account: Account<'info, Contributor>,
    #[account(
        mut,
        associated_token::mint=mint_to_raise,
        associated_token::authority=contributor,
        associated_token::token_program=token_program,
    )]
    pub contributor_ata:InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info>Contribute<'info> {
    pub fn contribute(&mut self,amount: u64)->Result<()>{
        require!(
            amount>1_u64.pow(self.mint_to_raise.decimals as u32),
            FundraiserError::ContributionTooSmall
        );

        require!(
            amount<=(self.fundraiser.amount_to_raise*MAX_CONTRIBUTION_PERCENTAGE/PERCENTAGE_SCALER),
            FundraiserError::ContributionTooBig
        );
        let current_time=Clock::get()?.unix_timestamp;
        require!(
            self.fundraiser.duration>=((current_time-self.fundraiser.start_time)/SECONDS_TO_DAYS)as u16,
            FundraiserError::FundraiserEnded
        );

        require!(
            (self.contributor_account.amount<=(
                self.fundraiser.amount_to_raise*MAX_CONTRIBUTION_PERCENTAGE/PERCENTAGE_SCALER
            ))&&((self.contributor_account.amount+amount)<=(
                self.fundraiser.amount_to_raise*MAX_CONTRIBUTION_PERCENTAGE/PERCENTAGE_SCALER
            )),FundraiserError::MaxContributionReached
        );

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.contributor_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.contributor.to_account_info(),
            mint: self.mint_to_raise.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_context, amount, self.mint_to_raise.decimals)?;

        Ok(())
    }
}