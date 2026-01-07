use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use std::str::FromStr;

mod public;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("BBLMZ2CV1qdiC7joGBgTEy4oS36dkJqvBWtBhh6XLAG2");

const SYSTEM_OWNER: &str = "nVyjcxVWTddPkUF74VbrfByNKyGf2m3Kv4fmyQJNiyi";
// const SYSTEM_PUBKEY: [u8; 32] = [
//    11, 167, 230, 243,  52,  58, 151, 249,
//    63, 219, 156, 241,  19, 112,   4, 105,
//    49,  50,  29,  85, 166, 126, 184,  27,
//   223,  23,  62,  34, 138,  60, 160,  85
// ];
//3333333333333333f64
//0.0055555555555555f64
const NUM_1_DIV_180: f64			= 0.0833333333333333f64;	// 1 / VAULT_RELEASE_DAY
const VAULT_RELEASE_DAY: i64		= 12;	// The number of days for which the Vault Pool is released
const REDEEM_RELEASE_180_SEC: i64	= 12 * 86400;
const REDEEM_RELEASE_90_SEC: i64	= 6 * 86400;
const REDEEM_RELEASE_30_SEC: i64	= 2 * 86400;

#[program]
mod redeem {
    use super::*;
	
	pub fn config(ctx: Context<Config>, params: ConfigParams) -> Result<()> {
		let all_config			= &mut ctx.accounts.all_config;

		all_config.allow_redeem	= params.allow_redeem;
		all_config.es_neb		= ctx.accounts.es_neb.key();
		all_config.neb			= ctx.accounts.neb.key();
		all_config.decimals		= 10u64.pow(params.decimals as u32);
		Ok(())
	}
	
	pub fn update_black_list(ctx : Context<UpdateBlackList>, block_this_user: bool) -> Result<()> {
        ctx.accounts.user_black_info.need_block = block_this_user;
        Ok(())
    }
	
	pub fn config_stake(ctx: Context<ConfigStake>, amount: u64) -> Result<()> {
		let all_config	= &mut ctx.accounts.all_config;
		
		public::transfer_to(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer.to_account_info(),
			amount * all_config.decimals,
		)?;
		
		all_config.total_neb	+= amount;
		
		Ok(())
	}
	
	pub fn config_confiscate(ctx: Context<ConfigConfiscate>) -> Result<()> {
		
		let all_config	= &mut ctx.accounts.all_config;
		
		let all_config_seed_str = "all_config";

        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer = [&seeds[..]];
		
		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			all_config.total_neb * all_config.decimals,
		)?;
		
		all_config.total_neb = 0;
		
		Ok(())
	}
	
    pub fn redeem(ctx: Context<Redeem>, params: RedeemParams) -> Result<()> {

		let all_config			= & ctx.accounts.all_config;
		let user_redeem_info	= &mut ctx.accounts.user_redeem_info;
		let reward_array_account= &mut ctx.accounts.vault.reward_array;
		let all_vault_info		= &mut ctx.accounts.vault.all_vault_info;
		let user_vault_info		= &mut ctx.accounts.vault.user_vault_info;
	
		require!(all_config.allow_redeem, ErrorCode::SystemDenyRedeem);
		require!(ctx.accounts.vault.user_black_info.need_block == false, ErrorCode::UserHasBeenBlock);

		public::transfer_to(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			ctx.accounts.all_config_es_neb_account.to_account_info(),
			ctx.accounts.signer.to_account_info(),
			params.amount * all_config.decimals,
		)?;

		let clock			= Clock::get().unwrap();
        let now_time		= clock.unix_timestamp;
		
		let wait_time		:i64;
		let redeem_add_amount : u64;
		
        match params.redeem_type{
            1 => {  // 1 month, 30% vesting
				wait_time			= REDEEM_RELEASE_30_SEC;
				redeem_add_amount	= (params.amount as f64 * 0.3f64) as u64;
            },
            2 => {  // 3 month, 60% vesting
				wait_time			= REDEEM_RELEASE_90_SEC;
				redeem_add_amount	= (params.amount as f64 * 0.6f64) as u64;
            },
            3 => {  // 6 month, 100% vesting
				wait_time			= REDEEM_RELEASE_180_SEC;
				redeem_add_amount	= params.amount;
				
				vault_calc(reward_array_account, all_vault_info, user_vault_info, 0, params.amount as i64)?;
			},
            _ => {
                return err!(ErrorCode::SelectRedeemTypeNoSupport)
            }
        }
		//释放esNEB 大于结束时间全部释放 小于结束时间通过占比释放
		if user_redeem_info.ongoing_redeem != 0{
			if now_time >= user_redeem_info.end_time{
				user_redeem_info.wait_claim		+= user_redeem_info.ongoing_redeem;
				user_redeem_info.ongoing_redeem	= 0;
			}else{
				let new_wait_claim				= (user_redeem_info.ongoing_redeem as f64 * ((now_time - user_redeem_info.start_time) as f64 / (user_redeem_info.end_time - user_redeem_info.start_time) as f64)) as u64;
				user_redeem_info.wait_claim 	+= new_wait_claim;
				user_redeem_info.ongoing_redeem	-= new_wait_claim;
			}
		}

		user_redeem_info.ongoing_redeem	+= redeem_add_amount;
		user_redeem_info.start_time		= now_time;
		user_redeem_info.end_time		= now_time + wait_time;

		if params.redeem_type != 3{
			//更新要释放的NEB 分180天将释放的NEB注入奖励池中 通过数组结合取模运算 来获取更新的奖励因子 
			update_now_day_release(now_time, reward_array_account, all_vault_info)?;
			
			let per_day_release	= NUM_1_DIV_180 * (params.amount - redeem_add_amount) as f64;

			let reward_array	= crate::GET_ARRAY!(reward_array_account, u64, VAULT_RELEASE_DAY as usize);

			for i in 0..=(VAULT_RELEASE_DAY - 1) {
				reward_array[i as usize] += per_day_release as u64;
			}
			
			all_vault_info.now_day_release	+= per_day_release / 86400.0;
			
			reward_array[all_vault_info.day_index as usize] = 0;
			
			if all_vault_info.total_vault != 0{
				all_vault_info.s_now		+= all_vault_info.now_day_release * (now_time - all_vault_info.time_last) as f64 / all_vault_info.total_vault as f64;
			}else{
				all_vault_info.s_now		= 0f64;
			}
			
			all_vault_info.time_last		= now_time;
		}
		
		crate::log!("redeem", ctx.accounts.signer.key(), [user_redeem_info.ongoing_redeem, user_vault_info.vault_amount, user_vault_info.v_vault_amount]);
		
        Ok(())
    }
	
	pub fn redeem_claim(ctx: Context<RedeemClaim>) -> Result<()> {
		let all_config			= &mut ctx.accounts.all_config;
		let user_redeem_info	= &mut ctx.accounts.user_redeem_info;
		let reward_array_account= &mut ctx.accounts.vault.reward_array;
		let all_vault_info		= &mut ctx.accounts.vault.all_vault_info;
		let user_vault_info		= &mut ctx.accounts.vault.user_vault_info;
		
		require!(all_config.allow_redeem, ErrorCode::SystemDenyRedeem);
		require!(ctx.accounts.vault.user_black_info.need_block == false, ErrorCode::UserHasBeenBlock);
		
		let clock				= Clock::get().unwrap();
		let now_time			= clock.unix_timestamp;
		// 计算出claim的代币数量
		if user_redeem_info.ongoing_redeem != 0{
			
			if now_time >= user_redeem_info.end_time{
				user_redeem_info.wait_claim		+= user_redeem_info.ongoing_redeem;
				user_redeem_info.ongoing_redeem	= 0;
			}else{
				let new_wait_claim				= (user_redeem_info.ongoing_redeem as f64 * ((now_time - user_redeem_info.start_time) as f64 / (user_redeem_info.end_time - user_redeem_info.start_time) as f64)) as u64;
				user_redeem_info.wait_claim 	+= new_wait_claim;
				user_redeem_info.ongoing_redeem	-= new_wait_claim;		
			}
		}
		//计算第三种释放方式中相当于质押的代币数量
		if user_vault_info.v_vault_amount != 0{
			if user_vault_info.v_vault_amount > user_redeem_info.wait_claim{
				vault_calc(reward_array_account, all_vault_info, user_vault_info, 0, -(user_redeem_info.wait_claim as i64))?;
			}else{
				vault_calc(reward_array_account, all_vault_info, user_vault_info, 0, -(user_vault_info.v_vault_amount as i64))?;
			}
		}
		
		let all_config_seed_str = "all_config";

        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer = [&seeds[..]];
		
		let wait_claim = user_redeem_info.wait_claim;
		
		require!(all_config.total_neb >= wait_claim, ErrorCode::NEBNoEnough);

		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			wait_claim * all_config.decimals,
		)?;
		
		all_config.total_neb		-= wait_claim;
		user_redeem_info.start_time	= now_time;
		user_redeem_info.wait_claim	= 0;
		
		crate::log!("redeem_claim", ctx.accounts.signer.key(), [user_redeem_info.ongoing_redeem, wait_claim, user_vault_info.vault_amount, user_vault_info.v_vault_amount]);
		
		Ok(())
	}

	pub fn vault(ctx: Context<Vault>, amount: u64) -> Result<()> {
		let all_config			= &mut ctx.accounts.all_config;
		let reward_array		= &mut ctx.accounts.vault.reward_array;
		let all_vault_info		= &mut ctx.accounts.vault.all_vault_info;
		let user_vault_info		= &mut ctx.accounts.vault.user_vault_info;
		
		require!(all_config.allow_redeem, ErrorCode::SystemDenyRedeem);
		require!(ctx.accounts.vault.user_black_info.need_block == false, ErrorCode::UserHasBeenBlock);
		
		public::transfer_to(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer.to_account_info(),
			amount * all_config.decimals,
		)?;
		
		vault_calc(reward_array, all_vault_info, user_vault_info, amount as i64, 0)?;
		
		crate::log!("vault", ctx.accounts.signer.key(), [user_vault_info.vault_amount, user_vault_info.v_vault_amount]);
		
		Ok(())
	}
	
	pub fn unvault(ctx: Context<Vault>) -> Result<()> {
		let all_config			= &mut ctx.accounts.all_config;
		let reward_array_account= &mut ctx.accounts.vault.reward_array;
		let all_vault_info		= &mut ctx.accounts.vault.all_vault_info;
		let user_vault_info		= &mut ctx.accounts.vault.user_vault_info;
		require!(all_config.allow_redeem, ErrorCode::SystemDenyRedeem);
		require!(ctx.accounts.vault.user_black_info.need_block == false, ErrorCode::UserHasBeenBlock);

		let unvault_amount = user_vault_info.vault_amount;

		vault_calc(reward_array_account, all_vault_info, user_vault_info, -(user_vault_info.vault_amount as i64), 0)?;

		let all_config_seed_str = "all_config";
        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer = [&seeds[..]];
		
		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			unvault_amount * all_config.decimals,
		)?;
	
		crate::log!("unvault", ctx.accounts.signer.key(), [user_vault_info.vault_amount, user_vault_info.v_vault_amount]);
	
		Ok(())
	}
	
	pub fn vault_claim(ctx: Context<Vault>) -> Result<()> {
		let all_config			= &mut ctx.accounts.all_config;
		let reward_array		= &mut ctx.accounts.vault.reward_array;
		let all_vault_info		= &mut ctx.accounts.vault.all_vault_info;
		let user_vault_info		= &mut ctx.accounts.vault.user_vault_info;
		require!(all_config.allow_redeem, ErrorCode::SystemDenyRedeem);
		require!(ctx.accounts.vault.user_black_info.need_block == false, ErrorCode::UserHasBeenBlock);
		
		vault_calc(reward_array, all_vault_info, user_vault_info, 0, 0)?;
		
		let all_config_seed_str = "all_config";
        let seeds	= &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer	= [&seeds[..]];
		
		let wait_claim = user_vault_info.wait_claim;

		require!(all_config.total_neb >= wait_claim, ErrorCode::NEBNoEnough);

		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_neb_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			wait_claim * all_config.decimals,
		)?;
		
		all_config.total_neb		-= wait_claim;
		user_vault_info.wait_claim = 0;
		
		crate::log!("vault_claim", ctx.accounts.signer.key(), [wait_claim, user_vault_info.vault_amount, user_vault_info.v_vault_amount]);
		
		Ok(())
	}
}

fn update_now_day_release(
	now_time			: i64, 
	reward_array_account: &mut AccountInfo,
	all_vault_info		: &mut Account<AllVaultInfo>,
) -> Result<()> {
	
	let now_day_index	= ((now_time / 86400) % VAULT_RELEASE_DAY) as u32;
	
	if all_vault_info.day_index != now_day_index{
		let duration: u32;
		
		if all_vault_info.day_index < now_day_index{
			duration	= now_day_index - all_vault_info.day_index;
		}else{
			duration	= VAULT_RELEASE_DAY as u32 - all_vault_info.day_index + now_day_index;
		}
		
		let mut sum_of_release	= 0u64;
		let mut i: u32			= all_vault_info.day_index + 1;
		
		let reward_array		= crate::GET_ARRAY!(reward_array_account, u64, VAULT_RELEASE_DAY as usize);
		
		loop {
			sum_of_release += reward_array[(i % VAULT_RELEASE_DAY as u32) as usize];
			reward_array[(i % VAULT_RELEASE_DAY as u32) as usize]	= 0;
			i += 1;
			if i > all_vault_info.day_index + duration{
				break;
			}
		}
		
		all_vault_info.now_day_release	= sum_of_release as f64 / 86400.0;
		all_vault_info.day_index		= now_day_index;
		
		if all_vault_info.total_vault != 0{
			all_vault_info.s_now		+= all_vault_info.now_day_release * (now_time - all_vault_info.time_last) as f64 / all_vault_info.total_vault as f64;
		}else{
			all_vault_info.s_now		= 0f64;
		}
		
		all_vault_info.time_last		= now_time;
	}
	
	Ok(())
}

fn vault_calc(
	reward_array_account: &mut AccountInfo,
	all_vault_info		: &mut Account<AllVaultInfo>,
	user_vault_info		: &mut Account<UserVaultInfo>,
	amount				: i64,
	v_amount			: i64,
) -> Result<()> {
	let now_time		= public::now_time_s();
	//更新线性释放的奖励 计算出释放部分的质押奖励
	update_now_day_release(now_time, reward_array_account, all_vault_info)?;
	//计算待领取的质押奖励 待领取的质押奖励 = (当前质押数量 + 当前redeem数量) * (当前奖励因子 -上一次奖励因子 )
	//先结算，再变更仓位 结算之前不将质押数量从池子中减出会导致delegate记录的奖励因子变小 从而导致领取的质押奖励变大 多给了一段时间的质押奖励
	if user_vault_info.vault_amount != 0 || user_vault_info.v_vault_amount != 0{
		all_vault_info.s_now		+= all_vault_info.now_day_release * (now_time - all_vault_info.time_last) as f64 / all_vault_info.total_vault as f64;
		all_vault_info.time_last	= now_time;
		all_vault_info.total_vault	-= user_vault_info.vault_amount + user_vault_info.v_vault_amount;
		user_vault_info.wait_claim	+= ((user_vault_info.vault_amount + user_vault_info.v_vault_amount) as f64 * (all_vault_info.s_now - user_vault_info.s_delegator)) as u64;
	}
	//更新当前的奖励
	if all_vault_info.total_vault != 0{
		all_vault_info.s_now		+= all_vault_info.now_day_release * (now_time - all_vault_info.time_last) as f64 / all_vault_info.total_vault as f64;
	}else{
		all_vault_info.s_now		= 0f64;
	}
	all_vault_info.time_last		= now_time;
	
	user_vault_info.s_delegator		= all_vault_info.s_now;
	
	require!((user_vault_info.vault_amount as i64 + amount) >= 0, ErrorCode::CalcNegative);
	require!((user_vault_info.v_vault_amount as i64 + v_amount) >= 0, ErrorCode::CalcNegative);
	
	user_vault_info.vault_amount	= (user_vault_info.vault_amount as i64 + amount) as u64;
	user_vault_info.v_vault_amount	= (user_vault_info.v_vault_amount as i64 + v_amount) as u64;
	
	all_vault_info.total_vault		+= user_vault_info.vault_amount + user_vault_info.v_vault_amount;
	
	Ok(())
}

/******************************************************
    STRUCT
******************************************************/ 

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ConfigParams {
	pub allow_redeem	: bool,
	pub decimals		: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RedeemParams {
    pub amount			: u64,
    pub redeem_type		: u32,
}

/******************************************************
    CONTEXT
******************************************************/ 

#[derive(Accounts)]
pub struct Config<'info> {
	#[account(
		init_if_needed,
		seeds = [b"all_config"],
		bump,
		payer = signer,
		space = 8 + 1 + 32 * 2 + 8 + 8,
	)]
	pub all_config			: Account<'info, AllConfig>,
	
	#[account(mut)]
    pub es_neb				: Account<'info, Mint>,
	
	#[account(mut)]
    pub neb					: Account<'info, Mint>,
	
	#[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer				: Signer<'info>,
    pub system_program		: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBlackList<'info> {
	///CHECK:black address 
    pub black_addr          : UncheckedAccount<'info>,

    #[account(init_if_needed, seeds = [b"black_list", black_addr.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     : Account<'info, BlackList>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub system_program      : Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfigStake<'info> {
	#[account(address = all_config.neb)]
    pub neb					: Account<'info, Mint>,

	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = all_config,
    )]
    pub all_config_neb_account	: Account<'info, TokenAccount>,

	#[account(mut, seeds = [b"all_config"], bump)]
	pub all_config			: Account<'info, AllConfig>,

	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub system_program      : Program<'info, System>,
	pub token_program       : Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ConfigConfiscate<'info> {
	#[account(mut, address = all_config.neb)]
    pub neb					: Account<'info, Mint>,

	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = all_config,
    )]
    pub all_config_neb_account	: Account<'info, TokenAccount>,

	#[account(mut, seeds = [b"all_config"], bump)]
	pub all_config			: Account<'info, AllConfig>,

	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub system_program      : Program<'info, System>,
	pub token_program       : Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}
//质押信息
#[derive(Accounts)]
pub struct VaultInfo<'info> {
	//黑名单用户记录
	#[account(
		init_if_needed, seeds = [b"black_list", 
		signer.key().as_ref()],
		bump,
		payer = signer,
		space = 8 + 1
	)]
    pub user_black_info     : Account<'info, BlackList>,
	//奖励数组槽位
	///CHECK:array account check
	#[account(
        init_if_needed,
        seeds = [b"reward_array"], 
        bump,
        payer = signer,
        space = 8 + 8 * 180
    )]
	pub reward_array		: AccountInfo<'info>,
	//全局质押变量信息
	#[account(
        init_if_needed,
        seeds = [b"all_v_info"], 
        bump,
        payer = signer,
        space = 8 + 8 + 8 + 8 + 8 + 4
    )]
	pub all_vault_info		: Account<'info, AllVaultInfo>,
	//用户质押信息
	#[account(
		init_if_needed,
		seeds = [b"user_v_info", signer.key().as_ref()], 
		bump,
		payer = signer,
		space = 8 + 8 + 8 + 8 + 8
	)]
	pub user_vault_info		: Account<'info, UserVaultInfo>,
	
	#[account(mut)]
	pub signer				: Signer<'info>,
    pub system_program		: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
	#[account(
		init_if_needed,
        payer = signer,
        associated_token::mint      = es_neb,
        associated_token::authority = all_config,
    )]
    pub all_config_es_neb_account	: Account<'info, TokenAccount>,

	#[account(seeds = [b"all_config"], bump)]
	pub all_config			: Account<'info, AllConfig>,

	#[account(address = all_config.es_neb)]
    pub es_neb				: Account<'info, Mint>,
	
	pub vault				: VaultInfo<'info>,
	
    #[account(
		init_if_needed, 
		seeds = [b"redeem", signer.key().as_ref()],
		bump,
		payer = signer, 
		space = 8 + 8 + 8 + 8 + 8)]
    pub user_redeem_info	: Account<'info, UserRedeemInfo>,
	
	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = es_neb,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
	
    #[account(mut)]
    pub signer				: Signer<'info>,
    pub system_program		: Program<'info, System>,
	pub token_program       : Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct RedeemClaim<'info> {

	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = all_config,
    )]
    pub all_config_neb_account	: Account<'info, TokenAccount>,

	#[account(mut, seeds = [b"all_config"], bump)]
	pub all_config			: Account<'info, AllConfig>,

	pub vault				: VaultInfo<'info>,

	#[account(address = all_config.neb)]
    pub neb					: Account<'info, Mint>,
	
	#[account(
		init_if_needed, 
		seeds = [b"redeem", signer.key().as_ref()],
		bump,
		payer = signer, 
		space = 8 + 8 + 8 + 8 + 8)]
    pub user_redeem_info	: Account<'info, UserRedeemInfo>,
	
	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
	
	#[account(mut)]
    pub signer				: Signer<'info>,
    pub system_program		: Program<'info, System>,
	pub token_program       : Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Vault<'info>{
	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = all_config,
    )]
    pub all_config_neb_account	: Account<'info, TokenAccount>,

	#[account(seeds = [b"all_config"], bump)]
	pub all_config			: Account<'info, AllConfig>,

	#[account(address = all_config.neb)]
    pub neb					: Account<'info, Mint>,
	
	pub vault				: VaultInfo<'info>,
	
	#[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = neb,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
	
    #[account(mut)]
    pub signer				: Signer<'info>,
    pub system_program		: Program<'info, System>,
	pub token_program       : Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}

/******************************************************
    ACCOUNT
******************************************************/ 

#[account]
pub struct AllConfig {
	pub allow_redeem	: bool,
	pub es_neb			: Pubkey,
	pub neb				: Pubkey,
	pub decimals		: u64,
	pub total_neb		: u64,
}

#[account]
pub struct BlackList {
    pub need_block		: bool,
}

#[account]
pub struct UserRedeemInfo {
	pub wait_claim		: u64,
    pub ongoing_redeem	: u64,
	pub start_time		: i64,
	pub end_time		: i64,
}

#[account]
pub struct AllVaultInfo {
	pub s_now			: f64,
	pub time_last		: i64,
	pub total_vault		: u64,
	pub now_day_release	: f64,
	pub day_index		: u32,
}

#[account]
pub struct UserVaultInfo{
	pub vault_amount	: u64,
	pub s_delegator		: f64,
	pub v_vault_amount	: u64,
	pub wait_claim		: u64,
}

#[error_code]
pub enum ErrorCode {
	#[msg("This user has been blocked.")]
    UserHasBeenBlock,
    #[msg("The system cannot allow redemption now.")]
    SystemDenyRedeem,
	#[msg("This Reddem type no support.")]
	SelectRedeemTypeNoSupport,
	#[msg("System neb no enough.")]
	NEBNoEnough,
	#[msg("There is an unexpected error.")]
	Unexpected,
	#[msg("Amount result is negative.")]
	CalcNegative,
}

