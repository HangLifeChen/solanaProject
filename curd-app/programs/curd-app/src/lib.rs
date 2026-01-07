use anchor_lang::prelude::*;

declare_id!("9nUPAnL2GsrLZAwa4HiGbJU4y2e8eRepKJgjKfhw4Twr");
#[program]
pub mod curd_app {
    use super::*;

    pub fn create_journal_entry(ctx: Context<CreateEntry>,title:String,content:String) -> Result<()> {
        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.owner = ctx.accounts.owner.key();
        journal_entry.title = title;
        journal_entry.content = content;
        Ok(())
    }

    pub fn update_journal_entry(ctx: Context<UpdateEntry>, title: String, content: String) -> Result<()> {
        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.title = title;
        journal_entry.content = content;
        Ok(())
    }

    pub fn delete_journal_entry(ctx: Context<DeleteEntry>,title: String) -> Result<()> {
        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.content = String::new();
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(title:String)]
pub struct CreateEntry<'info> {
    #[account(
        init,
        payer = owner,
        seeds=[title.as_bytes(),owner.key().as_ref()],
        bump,
        space = 8 + JournalEntryState::INIT_SPACE)]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title:String)]
pub struct DeleteEntry<'info> {
    #[account(
        mut,
        has_one = owner,
        seeds=[title.as_bytes(),owner.key().as_ref()],
        bump,
        close=owner
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title:String)]
pub struct UpdateEntry<'info> {
    #[account(
        mut, 
        has_one = owner,
        seeds=[title.as_bytes(),owner.key().as_ref()],
        bump,
        realloc=8+JournalEntryState::INIT_SPACE,
        realloc::payer=owner,
        realloc::zero=true,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}



#[account]
#[derive(InitSpace)]
pub struct JournalEntryState {
    pub owner: Pubkey,
    #[max_len(50)]
    pub title: String,
    #[max_len(1000)]
    pub content: String,
}
