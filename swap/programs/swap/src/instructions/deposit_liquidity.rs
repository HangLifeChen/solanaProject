use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::{mint_to, transfer_checked, MintTo, Token2022, TransferChecked}, token_interface::{Mint, TokenAccount}};
use fixed::types::I64F64;

use crate::{constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY}, errors::TutorialError, state::Pool};

pub fn deposit_liquidity(
    ctx:Context<DepositLiquidity>,
    amount_a:u64,
    amount_b:u64,
)->Result<()>{
    // 确保不会存入超过用户拥有的代币
    let mut amount_a = amount_a.min(ctx.accounts.depositor_account_a.amount);
    let mut amount_b = amount_b.min(ctx.accounts.depositor_account_b.amount);

    // 判断是否为第一次创建池子
    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;
    let pool_creation = pool_a.amount == 0 && pool_b.amount == 0;

    // 保证存入的比例和池子保持一致
    (amount_a, amount_b) = if pool_creation {
        (amount_a, amount_b)
    } else {
        let ratio = I64F64::from_num(pool_a.amount)
            .checked_div(I64F64::from_num(pool_b.amount))
            .unwrap();

        // 如果 a 偏多，则调整 a
        if I64F64::from_num(amount_a) > I64F64::from_num(amount_b) * ratio {
            (
                (I64F64::from_num(amount_b) * ratio).to_num::<u64>(),
                amount_b,
            )
        } else {
            (
                amount_a,
                (I64F64::from_num(amount_a) / ratio).to_num::<u64>(),
            )
        }
    };

    // 计算要 mint 的流动性代币数量
    let mut liquidity = I64F64::from_num(amount_a)
        .checked_mul(I64F64::from_num(amount_b))
        .unwrap()
        .sqrt()
        .to_num::<u64>();

    // 如果是首次创建池子，需要锁定 minimum_liquidity
    if pool_creation {
        if liquidity < MINIMUM_LIQUIDITY {
            return err!(TutorialError::DepositTooSmall);
        }
        liquidity -= MINIMUM_LIQUIDITY;
    }

  transfer_checked(
        CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.depositor_account_a.to_account_info(),
            to: ctx.accounts.pool_account_a.to_account_info(),
            mint: ctx.accounts.mint_a.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_a,
        6,
    )?;

      transfer_checked(
        CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.depositor_account_b.to_account_info(),
            to: ctx.accounts.pool_account_b.to_account_info(),
            mint: ctx.accounts.mint_b.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_b,
        6,
    )?;

    let authority_bump = ctx.bumps.pool_authority;
    let binding_a = ctx.accounts.mint_a.key();
    let binding_b = ctx.accounts.mint_b.key();
    let binding = [authority_bump];
    let signer_seeds = &[&[
        ctx.accounts.pool.amm.as_ref(),
        binding_a.as_ref(),
        binding_b.as_ref(),
        AUTHORITY_SEED,
        &binding,
    ][..]];



    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_liquidity.to_account_info(),
                to: ctx.accounts.depositor_account_liquidity.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds
        ),
        liquidity
    )?;

    Ok(())
}


#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(
        seeds = [
            pool.amm.as_ref(),
            pool.mint_a.key().as_ref(),
            pool.mint_b.key().as_ref(),
        ],
        bump,
        has_one = mint_a,
        has_one = mint_b,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Read only authority
    #[account(
        seeds = [
            pool.amm.as_ref(),
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            AUTHORITY_SEED,
        ],
        bump,
    )]
    pub pool_authority: AccountInfo<'info>,

    /// The account paying for all rents
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [
            pool.amm.as_ref(),
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            LIQUIDITY_SEED,
        ],
        bump,
    )]
    pub mint_liquidity: Box<InterfaceAccount<'info, Mint>>,

    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program,
    )]
    pub pool_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program,
    )]
    pub pool_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_liquidity,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_account_liquidity: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_account_b: InterfaceAccount<'info, TokenAccount>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}