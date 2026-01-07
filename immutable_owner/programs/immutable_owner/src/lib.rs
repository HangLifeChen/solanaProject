use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::token_2022;
use anchor_spl::token_interface::TokenAccount;
use anchor_spl::{
    token_2022::{
        initialize_account3,
        spl_token_2022::{extension::ExtensionType, pod::PodAccount, instruction::AuthorityType},
        InitializeAccount3,
        SetAuthority,
        TransferChecked
    },
    token_interface::{immutable_owner_initialize, ImmutableOwnerInitialize, Mint, Token2022},
};
declare_id!("CQvTZMwEHncKzauZTrzgwzeWf3yBtxdhmNuYVQtwKnBc");

#[program]
pub mod immutable_owner {


    use super::*;

    // There is currently not an anchor constraint to automatically initialize the ImmutableOwner extension
    // We can manually create and initialize the token account via CPIs in the instruction handler
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Calculate space required for token and extension data
        let token_account_size = ExtensionType::try_calculate_account_len::<PodAccount>(&[
            ExtensionType::ImmutableOwner,
        ])?;

        // Calculate minimum lamports required for size of token account with extensions
        let lamports = (Rent::get()?).minimum_balance(token_account_size);

        // Invoke System Program to create new account with space for token account and extension data
        create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                },
            ),
            lamports,                          // Lamports
            token_account_size as u64,         // Space
            &ctx.accounts.token_program.key(), // Owner Program
        )?;

        // Initialize the token account with the immutable owner extension
        immutable_owner_initialize(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            ImmutableOwnerInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                token_account: ctx.accounts.token_account.to_account_info(),
            },
        ))?;

        // Initialize the standard token account data
        initialize_account3(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            InitializeAccount3 {
                account: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ))?;
        
        Ok(())
    }

    pub fn transfer_ownership(ctx: Context<TransferOwnership>, new_owner: Pubkey) -> Result<()> {
        // Ensure the current owner is the signer
        require!(ctx.accounts.current_owner.is_signer, ErrorCode::Unauthorized);
        if ctx.accounts.current_owner.key() != *(ctx.accounts.account.owner) {
            return Err(ErrorCode::Unauthorized.into())
        }

        // Transfer ownership by setting the new owner
        let account_info = ctx.accounts.account.to_account_info();
        account_info.assign(&new_owner);

        Ok(())
    }

    pub fn change_token_owner(ctx: Context<ChangeTokenOwner>) -> Result<()> {
        // 转移代币账户所有权
        let cpi_accounts = SetAuthority {
            account_or_mint: ctx.accounts.token_account.to_account_info(),
            current_authority: ctx.accounts.current_owner.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts
        );

        anchor_spl::token_2022::set_authority(
            cpi_ctx,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.new_owner.key())
        )?;
        
        Ok(())
    }

    pub fn initialize_pda(ctx: Context<InitializePda>) -> Result<()> {
        // 1. 创建PDA账户
        let pda_account = &mut ctx.accounts.pda_account;
        pda_account.authority = ctx.accounts.authority.key();
        pda_account.data = 1; // 初始化数据字段
        Ok(())
    }

    pub fn transfer_pda_control(ctx: Context<TransferPdaControl>, new_authority: Pubkey) -> Result<()> {
        // 1. 验证当前权限
        require!(ctx.accounts.pda_account.authority == ctx.accounts.authority.key(), 
                ErrorCode::Unauthorized);
        
        // 2. 更新PDA账户中的权限字段
        ctx.accounts.pda_account.authority = new_authority;
        
        Ok(())
    }

    pub fn transfer_nft(ctx: Context<TransferNft2022>) -> Result<()> {
        // 1. 验证NFT属于当前所有者
        require!(
            ctx.accounts.nft_account.owner == ctx.accounts.current_owner.key(),
            ErrorCode::Unauthorized
        );

        // 2. 转移所有权 (使用Token-2022的transfer_checked)
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.nft_account.to_account_info().clone(),
            mint: ctx.accounts.mint_account.to_account_info().clone(),
            to: ctx.accounts.new_owner_token_account.to_account_info().clone(),
            authority: ctx.accounts.current_owner.to_account_info().clone(),
        };
        
        let decimals = 0; // NFT通常使用0位小数
        
        anchor_spl::token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts
            ),
            1,      // NFT数量
            decimals // 小数位数
        )?;
        
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub token_account: Signer<'info>,
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(mut)]
    /// CHECK: 该账户必须是可变的
    pub account: AccountInfo<'info>,  // 要转移的账户
    
    #[account(mut)]
    pub current_owner: Signer<'info>, // 当前所有者（必须签名）
    
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not the owner")]
    Unauthorized,
}


#[derive(Accounts)]
pub struct ChangeTokenOwner<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(signer)]
    /// CHECK: 当前所有者必须是签名者
    pub current_owner: AccountInfo<'info>,
    
    /// CHECK: 新所有者不需要是签名者
    pub new_owner: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token2022>,
}

#[account]
pub struct PdaAccount {
    pub authority: Pubkey,  // 控制权字段
    pub data: u64,
}

#[derive(Accounts)]
pub struct InitializePda<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8, // PDA账户的空间：u64 + Pubkey
        seeds = [b"pda_account_a".as_ref()],
        bump
    )]
    pub pda_account: Account<'info, PdaAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferPdaControl<'info> {
    #[account(
        mut,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub pda_account: Account<'info, PdaAccount>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferNft2022<'info> {
    #[account(mut)]
    pub nft_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub new_owner_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(signer)]
    pub current_owner: Signer<'info>,
    
    pub mint_account: InterfaceAccount<'info, Mint>,
    
    #[account(address = token_2022::ID)]
    pub token_program: Program<'info, Token2022>,
}