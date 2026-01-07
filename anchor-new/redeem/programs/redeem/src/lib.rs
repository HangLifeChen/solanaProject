use anchor_lang::prelude::*;

declare_id!("JArVZspQY6fNbRzC8K4yQhPmpZPE4wHCEb1SJcsdw7N4");

#[program]
pub mod redeem {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
