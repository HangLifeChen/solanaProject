use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::extension::permanent_delegate::PermanentDelegate,
    token_interface::{
        spl_pod::optional_keys::OptionalNonZeroPubkey,
        spl_token_2022::{
            extension::{BaseStateWithExtensions, StateWithExtensions},
            state::Mint as MintState,
        },
        Mint, Token2022, TokenAccount,
    },
};

declare_id!("CfQKZWTzhuCCtNPNGDuJraidcFERX3vNroiMq6Ppw4XP");

#[program]
pub mod permanent_delegate {
    use anchor_spl::token_2022::{burn, transfer_checked, Burn};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.check_mint_data()?;
        Ok(())
    }

    pub fn burn_user_tokens(ctx: Context<BurnUserTokens>, amount: u64) -> Result<()> {
        // PermanentDelegate 执行 burn
        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.delegate.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!(
            "PermanentDelegate {} burned {} tokens from {}",
            ctx.accounts.delegate.key(),
            amount,
            ctx.accounts.user_token_account.key()
        );
        Ok(())
    }

    pub fn transfer_user_tokens(ctx: Context<TransferUserTokens>, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.delegate.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.mint.decimals,
        )?;
        msg!(
            "PermanentDelegate {} transferred {} tokens from {} to {}",
            ctx.accounts.delegate.key(),
            amount,
            ctx.accounts.from.key(),
            ctx.accounts.to.key()
        );
        Ok(())
    }


}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 2,
        mint::authority = payer,
        extensions::permanent_delegate::delegate = payer,
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnUserTokens<'info> {
    /// PermanentDelegate
    pub delegate: Signer<'info>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// 用户的代币账户 (不需要用户签名)
    #[account(mut, token::mint = mint)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct TransferUserTokens<'info> {
    /// PermanentDelegate
    pub delegate: Signer<'info>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// 来源用户代币账户
    #[account(mut, token::mint = mint)]
    pub from: InterfaceAccount<'info, TokenAccount>,

    /// 目标代币账户
    #[account(mut, token::mint = mint)]
    pub to: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}



// helper to check mint data, and demonstrate how to read mint extension data within a program
impl<'info> Initialize<'info> {
    pub fn check_mint_data(&self) -> Result<()> {
        let mint = &self.mint_account.to_account_info();
        let mint_data = mint.data.borrow();
        let mint_with_extension = StateWithExtensions::<MintState>::unpack(&mint_data)?;
        let extension_data = mint_with_extension.get_extension::<PermanentDelegate>()?;

        assert_eq!(
            extension_data.delegate,
            OptionalNonZeroPubkey::try_from(Some(self.payer.key()))?
        );

        msg!("{:?}", extension_data);
        Ok(())
    }
}