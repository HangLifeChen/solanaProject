use anchor_spl::{token_2022::Token2022, token_interface::{transfer_fee_set, Mint, TransferFeeSetTransferFee}};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateFee<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
}

pub fn process_update_fee(
    ctx: Context<UpdateFee>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<()> {
    let mint_account = &mut ctx.accounts.mint_account;
    let cpi_accounts=TransferFeeSetTransferFee{
        token_program_id: ctx.accounts.token_program.to_account_info(),
        mint: mint_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    transfer_fee_set(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts
        ),
        transfer_fee_basis_points,
        maximum_fee
    )?;

    msg!(
        "Updated fee to {} basis points with a maximum fee of {}",
        transfer_fee_basis_points,
        maximum_fee
    );

    Ok(())
}
