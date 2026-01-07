use anchor_lang::prelude::*;

declare_id!("d7ntiuwYFaqEcfhuEgAgtzCmREDPJ8EsZAYjagRgYM8");

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
