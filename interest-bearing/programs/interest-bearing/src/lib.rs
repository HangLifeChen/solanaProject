use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::{
    token_2022::{
        initialize_mint2,
        spl_token_2022::{
            extension::{
                interest_bearing_mint::InterestBearingConfig, BaseStateWithExtensions,
                ExtensionType, StateWithExtensions,
            },
            pod::PodMint,
            state::Mint as MintState,
        },
        InitializeMint2,
    },
    token_interface::{
        interest_bearing_mint_initialize, interest_bearing_mint_update_rate,
        spl_pod::optional_keys::OptionalNonZeroPubkey, InterestBearingMintInitialize,
        InterestBearingMintUpdateRate, Mint, Token2022,
    },
};
declare_id!("Fyn47HxumLYQ2S5bsTBBzZhycXAzsVxhNsk1vGQZRB8X");

#[program]
pub mod interest_bearing {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, rate: i16) -> Result<()> {
        // Calculate space required for mint and extension data
        let mint_size = ExtensionType::try_calculate_account_len::<PodMint>(&[
            ExtensionType::InterestBearingConfig,
        ])?;

        // Calculate minimum lamports required for size of mint account with extensions
        let lamports = (Rent::get()?).minimum_balance(mint_size);

        // Invoke System Program to create new account with space for mint and extension data
        create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.mint_account.to_account_info(),
                },
            ),
            lamports,                          // Lamports
            mint_size as u64,                  // Space
            &ctx.accounts.token_program.key(), // Owner Program
        )?;

        // Initialize the InterestBearingConfig extension
        // This instruction must come before the instruction to initialize the mint data
        interest_bearing_mint_initialize(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InterestBearingMintInitialize {
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                },
            ),
            Some(ctx.accounts.payer.key()),
            rate,
        )?;

        // Initialize the standard mint account data
        initialize_mint2(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InitializeMint2 {
                    mint: ctx.accounts.mint_account.to_account_info(),
                },
            ),
            2,                               // decimals
            &ctx.accounts.payer.key(),       // mint authority
            Some(&ctx.accounts.payer.key()), // freeze authority
        )?;

        check_mint_data(
            &ctx.accounts.mint_account.to_account_info(),
            &ctx.accounts.payer.key(),
        )?;
        Ok(())
    }

    pub fn update_rate(ctx: Context<UpdateRate>, rate: i16) -> Result<()> {

         // 验证权限（只有rate_authority可调用）
        check_mint_data(
            &ctx.accounts.mint_account.to_account_info(),
            &ctx.accounts.authority.key(),
        )?;

        let state = get_extension_data(&ctx.accounts.mint_account.to_account_info())?;

        // // 检查时间间隔（至少30天）
        // let clock = Clock::get()?;
        // require!(
        //     clock.unix_timestamp - i64::from(state.last_update_timestamp) >= 30 * 24 * 60 * 60,
        //     ErrorCode::RateChangeTooFrequent
        // );

        interest_bearing_mint_update_rate(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InterestBearingMintUpdateRate {
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    rate_authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            rate,
        )?;

  
        Ok(())
    }


     pub fn calculate_interest(
        ctx: Context<CalculateInterest>,
        principal: u64,       // 本金（原始数量，不含小数）
        start_time: i64,      // 开始时间戳（秒）
    ) -> Result<()> {
        let clock = Clock::get()?;
        // 获取代币账户的Mint状态
        let state = get_extension_data(&ctx.accounts.mint_account.to_account_info())?;


         // 取 mint 的 decimals
        let decimals = ctx.accounts.mint_account.decimals;

        // 时间差（秒 -> 年化）
        let time_elapsed = clock.unix_timestamp - i64::from(start_time);
        let years_elapsed = time_elapsed as f64 / (365.0 * 24.0 * 60.0 * 60.0);

        // 利率转成浮点 (bps -> rate)
        let rate = f64::from(i16::from(state.current_rate)) / 10_000.0;

        // 复利计算
        let accrued = (principal as f64) * E.powf(rate * years_elapsed);
        let interest = accrued - (principal as f64);

        // 考虑 decimals，保持精度
        let scaled_interest = (interest * 10f64.powi(decimals as i32)) as u64;
        ctx.accounts.rate_state.last_interest = scaled_interest;
        // ctx.accounts.rate_state.last_update = clock.unix_timestamp;
        ctx.accounts.rate_state.last_update = time_elapsed;
        ctx.accounts.rate_state.current_rate = i16::from(state.current_rate);
        ctx.accounts.rate_state.rate_authority = Option::<Pubkey>::from(state.rate_authority).unwrap_or_default();
        Ok(())
    }

    pub fn create_rate_state(ctx: Context<CreateRateState>) -> Result<()> {
        // 创建 RateState 账户
        let rate_state = &mut ctx.accounts.rate_state;
        rate_state.rate_authority = ctx.accounts.payer.key();
        rate_state.last_update = Clock::get()?.unix_timestamp;
        rate_state.current_rate = 0; // 初始利率为0
        rate_state.last_interest = 0; // 初始利息为0

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

fn check_mint_data(
    mint_account_info: &AccountInfo,  // 代币账户的AccountInfo
    authority_key: &Pubkey           // 预期的利率管理员公钥
) -> Result<()> {
    // 1. 获取账户数据的不可变引用
    let mint_data = mint_account_info.data.borrow();

    // 2. 解析带扩展的代币状态
    let mint_with_extension = StateWithExtensions::<MintState>::unpack(&mint_data)?;

    // 3. 提取InterestBearingConfig扩展数据
    let extension_data = mint_with_extension.get_extension::<InterestBearingConfig>()?;
    // 4. 验证利率管理员是否匹配预期    
    assert_eq!(
        extension_data.rate_authority,                     // 实际存储的管理员
        OptionalNonZeroPubkey::try_from(Some(*authority_key))? // 预期管理员
    );

    // 5. 打印扩展数据（调试用）
    msg!("extension_data: {:?}", extension_data);

    Ok(())
}

fn get_extension_data(mint_account_info: &AccountInfo) -> Result<InterestBearingConfig> {
    let mint_data = mint_account_info.data.borrow();
    let mint_with_extension = StateWithExtensions::<MintState>::unpack(&mint_data)?;
    let extension_data = mint_with_extension.get_extension::<InterestBearingConfig>()?;
    Ok(*extension_data)
}

// #[account]
// pub struct RateState {
//     pub rate_authority: Pubkey, // 关联的利率管理员地址
//     pub last_update: i64,     // 最后更新时间戳（秒）
//     pub current_rate: i16,    // 当前年化利率（基点）
//     pub last_interest: u64,   // 最近计算的利息（仅示例）
// }

#[derive(Accounts)]
pub struct CalculateInterest<'info> {
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub rate_state: Account<'info, RateState>,
}

#[account]
#[derive(InitSpace)]
pub struct RateState {
    pub rate_authority: Pubkey, // 关联的利率管理员地址
    pub last_update: i64,     // 最后更新时间戳（秒）
    pub current_rate: i16,    // 当前年化利率（基点）
    pub last_interest: u64,   // 最近计算的利息（仅示例）
}

#[derive(Accounts)]
pub struct CreateRateState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<RateState>()
    )]
    pub rate_state: Account<'info, RateState>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only rate authority can update the rate")]
    Unauthorized,
    #[msg("Rate can be updated at most once per month")]
    RateChangeTooFrequent,
}