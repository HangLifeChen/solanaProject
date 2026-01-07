use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::{constants::{AUTHORITY_SEED, LIQUIDITY_SEED}, errors::TutorialError, state::{Amm, Pool}};
mod constants;
mod errors;
mod state;

declare_id!("9hs6FLjNGt55idE8YqnV9GvcKWMEH3x9R24TsJ4nV1c");

#[program]
pub mod swap_example {
    use anchor_spl::token_2022::{mint_to, transfer_checked, MintTo, TransferChecked};
    use fixed::types::I64F64;

    use crate::constants::MINIMUM_LIQUIDITY;

    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
    let pool=&mut ctx.accounts.pool;
    pool.amm = ctx.accounts.amm.key();
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    Ok(())
    }

    pub fn create_amm(ctx: Context<CreateAmm>, id: Pubkey, fee: u16) -> Result<()> {
    let amm=&mut ctx.accounts.amm;
    amm.id=id;
    amm.fee=fee;
    amm.admin=ctx.accounts.admin.key();
    Ok(())
    }
    pub fn deposit_liquidity(
        ctx:Context<DepositLiquidity>,
        amount_a:u64,
        amount_b:u64,
    )->Result<()>{
        msg!("Depositing liquidity: {}, {},{}", amount_a, amount_b,ctx.accounts.pool.key());
   
    Ok(())
}


}


#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(
        seeds = [
            amm.id.as_ref()
        ],
        bump,
    )]
    pub amm: Box<Account<'info, Amm>>,

    #[account(
        init,
        payer = payer,
        space = Pool::LEN,
        seeds = [
            amm.key().as_ref(),
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
        ],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Read only authority
    #[account(
        seeds = [
            amm.key().as_ref(),
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            AUTHORITY_SEED,
        ],
        bump,
    )]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [
            amm.key().as_ref(),
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            LIQUIDITY_SEED,
        ],
        bump,
        mint::decimals = 6,
        mint::authority = pool_authority,
    )]
    pub mint_liquidity: Box<InterfaceAccount<'info, Mint>>,

    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority,
    )]
    pub pool_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = pool_authority,
    )]
    pub pool_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id:Pubkey,fee:u16)]
pub struct CreateAmm<'info> {
    #[account(
        init,
        payer = payer,
        space = Amm::LEN,
        seeds=[id.as_ref()],
        bump,
        constraint=fee <= 10000 @TutorialError::InvalidFee
    )]
    pub amm: Account<'info, Amm>,
    /// CHECK: Read only admin
    pub admin: AccountInfo<'info>,
     #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
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

    // #[account(
    //     init_if_needed,
    //     payer = payer,
    //     associated_token::mint = mint_liquidity,
    //     associated_token::authority = depositor,
    // )]
    // pub depositor_account_liquidity: Box<InterfaceAccount<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     associated_token::mint = mint_a,
    //     associated_token::authority = depositor,
    //     associated_token::token_program = token_program,
    // )]
    // pub depositor_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     associated_token::mint = mint_b,
    //     associated_token::authority = depositor,
    // )]
    // pub depositor_account_b: InterfaceAccount<'info, TokenAccount>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
