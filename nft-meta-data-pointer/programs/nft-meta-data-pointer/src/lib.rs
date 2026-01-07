use anchor_lang::prelude::*;

declare_id!("699Jc9QiuFZJpXPTaqfymX9Z9Pk1wvr4aZQXXqzyrdh1");

#[program]
pub mod nft_meta_data_pointer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
