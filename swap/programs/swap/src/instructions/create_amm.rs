use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
pub fn create_amm(ctx: Context<CreateAmm>, id: Pubkey, fee: u16) -> Result<()> {
    let amm=&mut ctx.accounts.amm;
    amm.id=id;
    amm.fee=fee;
    amm.admin=ctx.accounts.admin.key();
    Ok(())
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




