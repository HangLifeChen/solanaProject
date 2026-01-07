#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

// 地址要与配置文件一致
declare_id!("CrLNRtbdBgTez7g16PNr7G3ZwbGhZyFX26MttQiAgvV8");

#[program]
pub mod voting {
    use super::*;
    pub fn initialize_poll(ctx: Context<InitializePoll>,
                            poll_id:u64,
                            description:String,
                            poll_start:u64,
                            poll_end:u64) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.poll_id = poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        Ok(())
    }

    pub fn initialize_candidate(ctx: Context<InitializeCandidate>,
                            poll_id:u64,
                            candidate_name:String) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        let poll = &mut ctx.accounts.poll;
        poll.candidate_amount += 1;
        candidate.name = candidate_name;
        candidate.vote_count = 0;
        Ok(())
    }
    //Vote 结构体是 Anchor 要求的账户声明结构体，不是数据存储结构体。
    // 它只是一个 账户声明结构体，告诉 Anchor：
    // 这个指令需要用到哪些链上账户
    //如何验证这些账户
    pub fn vote(ctx: Context<Vote>,
               poll_id:u64,
               candidate_name:String) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        candidate.vote_count += 1;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate_name: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds=[poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        mut,
        seeds=[poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate: Account<'info, Candidate>,
}

#[derive(Accounts)]
// Anchor 要求 #[instruction(...)] 里参数顺序必须和指令函数的参数顺序完全一致，否则 seeds 生成时可能会错。
#[instruction(poll_id: u64,candidate_name:String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds=[poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll: Account<'info, Poll>,

    #[account(
        init,
        space = 8 + Candidate::INIT_SPACE,
        payer = signer,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    pub vote_count: u64,
    #[max_len(32)]
    pub name: String,
}


#[derive(Accounts)]
// 默认情况下，poll_id 只会传到函数里，而不会传到 Vote 这个账户验证结构体里
// 这个属性告诉anchor 这个账户上下文在验证时，会额外接收一个指令参数 poll_id（类型是 u64）
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    init,
    space = 8 + Poll::INIT_SPACE,
    payer = signer,
    seeds = [poll_id.to_le_bytes().as_ref()],
    bump,
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(32)]
    pub description: String,
    pub poll_start:u64,
    pub poll_end:u64,
    pub candidate_amount:u64,
}
