use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
 
declare_id!("68ekk2WeV5ip8pdL4wtqKDHAwWjHU5mbEtFFCAL2UyKi");
 
#[program]
pub mod transfer_sol {
    use super::*;
 


    pub fn sol_transfer(ctx: Context<SolTransfer>, amount: u64) -> Result<()> {
        let from_pubkey = ctx.accounts.sender.to_account_info();
        let to_pubkey = ctx.accounts.recipient.to_account_info();
        let program_id = ctx.accounts.system_program.to_account_info();
 
        let cpi_context = CpiContext::new(
            program_id,
            Transfer {
                from: from_pubkey,
                to: to_pubkey,
            },
        );

        emit!(MyEvent {
            user: ctx.accounts.sender.key(),
            amount: amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
            fee:0,
        });
        transfer(cpi_context, amount)?;
        Ok(())
    }


     // 带手续费的分级转账
    pub fn sol_transfer_with_fee(ctx: Context<SolTransferWithFee>, amount: u64) -> Result<()> {
        let fee = amount / 10; // 10%手续费
        
        // 主转账
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender.to_account_info(),
                    to: ctx.accounts.recipient.to_account_info(),
                },
            ),
            amount - fee,
        )?;

        // 手续费转账
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender.to_account_info(),
                    to: ctx.accounts.fee_receiver.to_account_info(),
                },
            ),
            fee,
        )?;
        emit!(MyEvent {
            user: ctx.accounts.sender.key(),
            amount: amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
            fee:fee,
        });
        Ok(())
    }
}

 
#[derive(Accounts)]
pub struct SolTransfer<'info> {
    #[account(mut)]
    sender: Signer<'info>,
    #[account(mut)]
    recipient: SystemAccount<'info>,
    system_program: Program<'info, System>,

    // 可选：手续费接收账户 等效于js--- .accounts({feeReceiver: new PublicKey("任意地址"),
    #[account(mut, address = "FEE_RECEIVER_PUBKEY".parse().unwrap())]
    /// CHECK:
    pub fee_receiver: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct SolTransferWithFee<'info> {
    #[account(mut)]
    sender: Signer<'info>,
    #[account(mut)]
    recipient: SystemAccount<'info>,
    system_program: Program<'info, System>,

    // 可选：手续费接收账户 等效于js--- .accounts({feeReceiver: new PublicKey("任意地址"),
    #[account(mut, address = "FEE_RECEIVER_PUBKEY".parse().unwrap())]
    /// CHECK:
    pub fee_receiver: AccountInfo<'info>
}

#[event]
pub struct MyEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub fee: u64,
}