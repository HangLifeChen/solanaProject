use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_interface::{spl_token_metadata_interface, Mint, Token2022};
use spl_token_metadata_interface::instruction::emit;

#[derive(Accounts)]
pub struct Emit<'info> {
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
}

// Invoke the emit instruction from spl_token_metadata_interface directly
// There is not an anchor CpiContext for this instruction
//更新元数据 → 触发emit → 索引服务捕获事件
pub fn process_emit(ctx: Context<Emit>) -> Result<()> {
    invoke(
        &emit(
            &ctx.accounts.token_program.key(), // token program id
            &ctx.accounts.mint_account.key(),  // "metadata" account
            None,
            None,
        ),
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint_account.to_account_info(),
        ],
    )?;
    Ok(())
}