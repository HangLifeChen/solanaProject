use anchor_lang::prelude::*;

declare_id!("AHtHF3DMGyjPV8KLqZ7MPBqEp4adZdW9J6bTaNzzBSAF");

#[program]
pub mod worker_verifier_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
