use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::{transfer_checked, Token2022, TransferChecked}, token_interface::{Mint, TokenAccount}};
use fixed::types::I64F64;


use crate::{
    constants::AUTHORITY_SEED,
    errors::*,
    state::{Amm, Pool},
};

pub fn swap_exact_tokens_for_tokens(
    ctx:Context<SwapExactTokensForTokens>,
    swap_a:bool,
    input_amount:u64,
    min_output_amount:u64,
)->Result<()>{
    let input = if swap_a&& input_amount>ctx.accounts.trader_account_a.amount {
        ctx.accounts.trader_account_a.amount
    } else if !swap_a && input_amount>ctx.accounts.trader_account_b.amount {
        ctx.accounts.trader_account_b.amount
    } else {
        input_amount
    };
    let amm=&ctx.accounts.amm;
    let taxed_input =input-input*amm.fee as u64/10000;

    let pool_a=&ctx.accounts.pool_account_a;
    let pool_b=&ctx.accounts.pool_account_b;
    let output =if swap_a {
       I64F64::from_num(taxed_input)
            .checked_mul(I64F64::from_num(pool_b.amount))
            .unwrap()
            .checked_div(
                I64F64::from_num(pool_a.amount).checked_add(
                    I64F64::from_num(taxed_input)
                ).unwrap()
            )
            .unwrap()
    } else {
       I64F64::from_num(taxed_input)
            .checked_mul(I64F64::from_num(pool_a.amount))
            .unwrap()
            .checked_div(
                I64F64::from_num(pool_b.amount).checked_add(
                    I64F64::from_num(taxed_input)
                ).unwrap()
            )
            .unwrap()
    }.to_num::<u64>();

    if output < min_output_amount {
        return err!(TutorialError::OutputTooSmall);
    }

    let invariant = I64F64::from_num(pool_a.amount)
        .checked_mul(I64F64::from_num(pool_b.amount))
        .unwrap().to_num::<u64>();
    let authority_bump = ctx.bumps.pool_authority;
    let binding_b = ctx.accounts.mint_b.key();
    let binding_a = ctx.accounts.mint_a.key();
    let authority_seeds = &[
        ctx.accounts.pool.amm.as_ref(),
        binding_a.as_ref(),
        binding_b.as_ref(),
        AUTHORITY_SEED,
        &[authority_bump],
    ];

    let signer = &[&authority_seeds[..]];

    if swap_a{
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked{
                    from:ctx.accounts.trader_account_a.to_account_info(),
                    mint:ctx.accounts.mint_a.to_account_info(),
                    to:ctx.accounts.pool_account_a.to_account_info(),
                    authority:ctx.accounts.trader.to_account_info(),
                },
            ),
            input,
            6
        )?;
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked{
                    from:ctx.accounts.pool_account_b.to_account_info(),
                    mint:ctx.accounts.mint_b.to_account_info(),
                    to:ctx.accounts.trader_account_b.to_account_info(),
                    authority:ctx.accounts.pool_authority.to_account_info(),
                },
                signer,
            ),
            output,
            6
        )?;
    }else {
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked{
                    from:ctx.accounts.trader_account_b.to_account_info(),
                    mint:ctx.accounts.mint_b.to_account_info(),
                    to:ctx.accounts.pool_account_b.to_account_info(),
                    authority:ctx.accounts.trader.to_account_info(),
                },
            ),
            input,
            6
        )?;
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked{
                    from:ctx.accounts.pool_account_a.to_account_info(),
                    mint:ctx.accounts.mint_a.to_account_info(),
                    to:ctx.accounts.trader_account_a.to_account_info(),
                    authority:ctx.accounts.pool_authority.to_account_info(),
                },
                signer,
            ),
            output,
            6
        )?;
    }

    msg!(
        "Traded {} tokens{}() after fees for {}",
        input,
        taxed_input,
        output
    );

    ctx.accounts.pool_account_a.reload()?;
    ctx.accounts.pool_account_b.reload()?;
    if invariant > ctx.accounts.pool_account_a.amount * ctx.accounts.pool_account_b.amount {
        return err!(TutorialError::InvariantViolated);
    }

    Ok(())
}


#[derive(Accounts)]
pub struct SwapExactTokensForTokens<'info>{
    #[account(
        seeds=[
            amm.id.as_ref(),
        ],
        bump,
    )]
    pub amm:Account<'info, Amm>,
    #[account(
        seeds=[
            pool.amm.as_ref(),
            pool.mint_a.key().as_ref(),
            pool.mint_b.key().as_ref(),
        ],
        bump,
        has_one=amm,
        has_one=mint_a,
        has_one=mint_b,
    )]
    pub pool:Account<'info, Pool>,
    ///CHECK: Read only authority
    #[account(
        seeds=[
            pool.amm.as_ref(),
            pool.mint_a.key().as_ref(),
            pool.mint_b.key().as_ref(),
            AUTHORITY_SEED
        ],
        bump,
    )]
    pub pool_authority:AccountInfo<'info>,

    pub trader:Signer<'info>,

    pub mint_a:Box<InterfaceAccount<'info, Mint>>,
    pub mint_b:Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint=mint_a,
        associated_token::authority=pool_authority,
        associated_token::token_program=token_program,
    )]
    pub pool_account_a:InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint=mint_b,
        associated_token::authority=pool_authority,
        associated_token::token_program=token_program,
    )]
    pub pool_account_b:InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer=payer,
        associated_token::mint=mint_a,
        associated_token::authority=trader,
        associated_token::token_program=token_program,
    )]
    pub trader_account_a:InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer=payer,
        associated_token::mint=mint_b,
        associated_token::authority=trader,
        associated_token::token_program=token_program,
    )]
    pub trader_account_b:InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub payer:Signer<'info>,
    pub token_program:Program<'info, Token2022>,
    pub associated_token_program:Program<'info, AssociatedToken>,
    pub system_program:Program<'info, System>,
}

