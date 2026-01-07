use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use solana_program::{
    ed25519_program,
    sysvar::{instructions::load_instruction_at_checked},
};
use std::str::FromStr;

mod public;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("JBpsmhjTYLkz5BVUywBVNAbT8oQXpDCu4nbXUhuMYLjF");

const SYSTEM_OWNER: &str = "8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1";
const SYSTEM_PUBKEY: [u8; 32] = [
  117, 193, 201, 226,  28, 226, 206,  95,
  180,   3,  43, 113,   2, 154, 252, 159,
  133, 166, 116,  31, 244, 165,  48, 182,
   61, 155, 116, 163, 227, 166, 196,   8
];

#[program]
mod worker_verifier_stake {
    use super::*;

    pub fn config(ctx : Context<Config>, config_params: InitSystemConfig) -> Result<()> {

        let all_config                      = &mut ctx.accounts.all_config;

        all_config.stake_token              = ctx.accounts.stake_token.key();
        all_config.token_decimals           = 10u64.pow(ctx.accounts.stake_token.decimals as u32);
        all_config.worker_unstake_lock_time  = config_params.worker_unstake_lock_time;
        all_config.verifier_unstake_lock_time= config_params.verifier_unstake_lock_time;
        all_config.delegator_unstake_lock_time= config_params.delegator_unstake_lock_time;
		all_config.worker_need_count        = config_params.worker_need_count;
        all_config.verifier_need_count_l1   = config_params.verifier_need_count_l1;
        all_config.verifier_need_count_l2   = config_params.verifier_need_count_l2;
        all_config.verifier_need_count_l3   = config_params.verifier_need_count_l3;
        all_config.verifier_need_count_l4   = config_params.verifier_need_count_l4;
        all_config.verifier_need_count_l5   = config_params.verifier_need_count_l5;
        all_config.allow_unstake             = config_params.allow_unstake;

		crate::log!("stake_config", ctx.accounts.signer.key(), [
			config_params.worker_unstake_lock_time, 
			config_params.verifier_unstake_lock_time, 
			config_params.delegator_unstake_lock_time,
			config_params.worker_need_count,
			config_params.verifier_need_count_l1,
			config_params.verifier_need_count_l2,
			config_params.verifier_need_count_l3,
			config_params.verifier_need_count_l4,
			config_params.verifier_need_count_l5
		]);
		
        Ok(())
    }

    pub fn add_black_list(ctx : Context<UpdateBlackList>) -> Result<()> {
        ctx.accounts.user_black_info.need_block = true;
        Ok(())
    }

    pub fn remove_black_list(ctx : Context<UpdateBlackList>) -> Result<()> {
        ctx.accounts.user_black_info.need_block = false;
        Ok(())
    }

	pub fn confiscate(ctx : Context<SystemConfiscate>) -> Result<()> {
		let all_config = &mut ctx.accounts.all_config;

		let all_config_seed_str = "all_config";
		let seeds	= &[
			all_config_seed_str.as_bytes(),
			&[ctx.bumps.all_config],
		];
		let signer	= [&seeds[..]];

		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			(all_config.worker_stake_num + all_config.verifier_stake_num + all_config.delegator_stake_num) * all_config.token_decimals,
		)?;

		all_config.worker_stake_num     = 0;
		all_config.verifier_stake_num   = 0;

		Ok(())
	}

    pub fn worker_stake(ctx : Context<Stake>, stake_amount: u64) -> Result<()> {
        require!(
            ctx.accounts.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );
		
		public::transfer_to(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			ctx.accounts.all_config_account.to_account_info(),
			ctx.accounts.signer.to_account_info(),
			stake_amount * ctx.accounts.all_config.token_decimals,
		)?;

        let all_config              = &mut ctx.accounts.all_config;
        all_config.worker_stake_num += stake_amount;

        let user_stake_info         = &mut ctx.accounts.user_stake_info;
        user_stake_info.w_total_stake_num           += stake_amount;
        user_stake_info.w_allow_online_device_num   = (user_stake_info.w_total_stake_num / all_config.worker_need_count) as u32;

		crate::log!("worker_stake", ctx.accounts.signer.key(), [user_stake_info.w_total_stake_num]);
        emit!(
            WorkerStakeEvent{
            signer: ctx.accounts.signer.key(),
            w_total_stake_num: user_stake_info.w_total_stake_num,
        });
        Ok(())
    }

    pub fn worker_unstake(ctx : Context<UnStake>, unstake_amount: u64) -> Result<()> {
        let user_stake_info		= &mut ctx.accounts.user_stake_info;
		let user_unstake_info	= &mut ctx.accounts.user_unstake_info;
		let all_config			= ctx.accounts.all_config.clone();
		
		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

		require!(
			unstake_amount <= user_stake_info.w_total_stake_num,
			ErrorCode::UnStakeToMuch
		);

		user_stake_info.w_total_stake_num			-= unstake_amount;
		user_stake_info.w_allow_online_device_num   = (user_stake_info.w_total_stake_num / all_config.worker_need_count) as u32;

        let now_time = public::now_time_s();
		
		if user_unstake_info.w_unstake_wait_time <= now_time{
			user_unstake_info.w_wait_claim_unstake	+= user_unstake_info.w_ongoing_unstake;
			user_unstake_info.w_ongoing_unstake		= 0;
		}
		
		user_unstake_info.w_unstake_wait_time = now_time + all_config.worker_unstake_lock_time;
		user_unstake_info.w_ongoing_unstake	+= unstake_amount;
	
		crate::log!("worker_unstake", ctx.accounts.signer.key(), [user_stake_info.w_total_stake_num, user_unstake_info.w_ongoing_unstake, now_time, user_unstake_info.w_wait_claim_unstake]);
        emit!(
            WorkerUnstakeEvent{
                signer: ctx.accounts.signer.key(),
                w_total_stake_num: user_stake_info.w_total_stake_num,
                w_ongoing_unstake: user_unstake_info.w_ongoing_unstake,
                now_time: now_time,
                w_wait_claim_unstake: user_unstake_info.w_wait_claim_unstake,
            }
        );
        Ok(())
    }

    pub fn verifier_stake(ctx : Context<VerifierStake>, params: VerifierStakeInfo) -> Result<()> {
		
		let stake_check			= &mut ctx.accounts.stake_check;
		
		require!(!stake_check.has_been_used, ErrorCode::StakeSigAlreadyUse);
        
		stake_check.has_been_used = true;
		
		let instruction_sysvar	= &ctx.accounts.instruction_sysvar;
        let current_index		= solana_program::sysvar::instructions::load_current_index_checked(instruction_sysvar)?;

        let ed25519_ix			= load_instruction_at_checked((current_index - 1).into(), instruction_sysvar)?;
        require_eq!(ed25519_ix.program_id, ed25519_program::id(), ErrorCode::SignatureMissing);

		let data				= &ed25519_ix.data;
		
		use std::convert::TryInto;

        let sig_offset			= u16::from_le_bytes(data[2..4].try_into().unwrap()) as usize;
        let pubkey_offset		= u16::from_le_bytes(data[6..8].try_into().unwrap()) as usize;
        let msg_offset			= u16::from_le_bytes(data[10..12].try_into().unwrap()) as usize;
        let msg_size			= u16::from_le_bytes(data[12..14].try_into().unwrap()) as usize;

		require!(data.len() >= sig_offset + 64, ErrorCode::SignatureDataLenErr);
        require!(data.len() >= pubkey_offset + 32, ErrorCode::SignatureDataLenErr);
        require!(data.len() >= msg_offset + msg_size, ErrorCode::SignatureDataLenErr);

		let sig_bytes			= &data[sig_offset..sig_offset + 64];
        let pubkey_bytes		= &data[pubkey_offset..pubkey_offset + 32];
        let message_bytes		= &data[msg_offset..msg_offset + msg_size];

        if sig_bytes != params.signature {
            return err!(ErrorCode::SignatureMissing);
        }
        if pubkey_bytes != SYSTEM_PUBKEY {
            return err!(ErrorCode::SignatureOwnerError);
        }
	
		let mut expected_message = vec![];
		expected_message.extend_from_slice(ctx.accounts.stake.signer.key.as_ref());
		expected_message.extend_from_slice(&params.stake_amount.to_le_bytes());
		expected_message.extend_from_slice(&params.nonce.to_le_bytes());
		expected_message.extend_from_slice(&params.timestamp.to_le_bytes());
		
        if message_bytes != expected_message.as_slice() {
            return err!(ErrorCode::SignatureDataErr);
        }

		require!(
            ctx.accounts.stake.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

		let all_config					= &mut ctx.accounts.stake.all_config;

		public::transfer_to(
			ctx.accounts.stake.token_program.to_account_info(),
			ctx.accounts.stake.signer_account.to_account_info(),
			ctx.accounts.stake.all_config_account.to_account_info(),
			ctx.accounts.stake.signer.to_account_info(),
			params.stake_amount * all_config.token_decimals,
		)?;
		
        all_config.verifier_stake_num   += params.stake_amount;

        let user_stake_info             = &mut ctx.accounts.stake.user_stake_info;
		let verifier_pool				= &mut ctx.accounts.verifier_pool;
		
		verifier_pool.delegator_num		+= params.stake_amount;
		verifier_pool.is_working		= true;
		
        user_stake_info.v_total_stake_num           += params.stake_amount;
        user_stake_info.v_allow_online_device_level = {
            let verifier_stake_num = user_stake_info.v_total_stake_num;
            if verifier_stake_num >= all_config.verifier_need_count_l5{
                5
            }else if verifier_stake_num >= all_config.verifier_need_count_l4{
                4
            }else if verifier_stake_num >= all_config.verifier_need_count_l3{
                3
            }else if verifier_stake_num >= all_config.verifier_need_count_l2{
                2
            }else if verifier_stake_num >= all_config.verifier_need_count_l1{
                1
            }else{
				verifier_pool.is_working = false;
                0
            }
        };
		
		crate::log!("verifier_stake", ctx.accounts.stake.signer.key(), [user_stake_info.v_total_stake_num, params.nonce]);
        emit!(
            VerifierStakeEvent{
                signer: ctx.accounts.stake.signer.key(),
                v_total_stake_num: user_stake_info.v_total_stake_num,
                nonce: params.nonce,
            }
        );
        Ok(())
    }

    pub fn verifier_unstake(ctx : Context<VerifierUnStake>, unstake_amount: u64) -> Result<()> {
        let user_stake_info		= &mut ctx.accounts.unstake.user_stake_info;
		let user_unstake_info	= &mut ctx.accounts.unstake.user_unstake_info;
		let all_config			= ctx.accounts.unstake.all_config.clone();
		let verifier_pool				= &mut ctx.accounts.verifier_pool;

		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.unstake.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

		require!(
			unstake_amount <= user_stake_info.v_total_stake_num,
			ErrorCode::UnStakeToMuch
		);

		verifier_pool.delegator_num			-= unstake_amount;
		verifier_pool.is_working			= true;

		user_stake_info.v_total_stake_num	-= unstake_amount;
		user_stake_info.v_allow_online_device_level = {
            let verifier_stake_num = user_stake_info.v_total_stake_num;
            if verifier_stake_num >= all_config.verifier_need_count_l5{
                5
            }else if verifier_stake_num >= all_config.verifier_need_count_l4{
                4
            }else if verifier_stake_num >= all_config.verifier_need_count_l3{
                3
            }else if verifier_stake_num >= all_config.verifier_need_count_l2{
                2
            }else if verifier_stake_num >= all_config.verifier_need_count_l1{
                1
            }else{
				verifier_pool.is_working	= false;
                0
            }
        };
		
		let now_time = public::now_time_s();
		
		if user_stake_info.v_allow_online_device_level == 0{
			ctx.accounts.verifier_pool.is_working	= false;
			ctx.accounts.verifier_pool.stop_time	= now_time;
		}
		
		if user_unstake_info.v_unstake_wait_time <= now_time{
			user_unstake_info.v_wait_claim_unstake	+= user_unstake_info.v_ongoing_unstake;
			user_unstake_info.v_ongoing_unstake		= 0;
		}
		
		user_unstake_info.v_unstake_wait_time = now_time + all_config.verifier_unstake_lock_time;
		user_unstake_info.v_ongoing_unstake	+= unstake_amount;

		crate::log!("verifier_unstake", ctx.accounts.unstake.signer.key(), [user_stake_info.v_total_stake_num, user_unstake_info.v_ongoing_unstake, now_time, user_unstake_info.v_wait_claim_unstake]);
        emit!(
            VerifierUnstakeEvent{
                signer: ctx.accounts.unstake.signer.key(),
                v_total_stake_num: user_stake_info.v_total_stake_num,
                v_ongoing_unstake: user_unstake_info.v_ongoing_unstake,
                now_time: now_time,
                v_wait_claim_unstake: user_unstake_info.v_wait_claim_unstake,
            }
        );
        Ok(())
    }
	
	pub fn delegator_stake(ctx : Context<DelegatorStake>, stake_amount: u64) -> Result<()> {
		require!(ctx.accounts.verifier_pool.is_working, ErrorCode::VerifierPoolStop);
		
		require!(
            ctx.accounts.stake.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );
		
		let all_config						= &mut ctx.accounts.stake.all_config;
		let delegate_info					= &mut ctx.accounts.delegate_info;
		let user_stake_info					= &mut ctx.accounts.stake.user_stake_info;
		
		public::transfer_to(
			ctx.accounts.stake.token_program.to_account_info(),
			ctx.accounts.stake.signer_account.to_account_info(),
			ctx.accounts.stake.all_config_account.to_account_info(),
			ctx.accounts.stake.signer.to_account_info(),
			stake_amount * all_config.token_decimals,
		)?;
		
		all_config.delegator_stake_num		+= stake_amount;

		
		if delegate_info.stake_amount == 0{
			ctx.accounts.verifier_pool.delegator_num += 1;
		}
		
		user_stake_info.d_total_stake_num	+= stake_amount;
		delegate_info.stake_amount			+= stake_amount;

		crate::log!("delegator_stake", ctx.accounts.stake.signer.key(), [user_stake_info.d_total_stake_num, delegate_info.stake_amount, ctx.accounts.pool_creator.to_account_info().key()]);
        emit!(
            DelegatorStakeEvent{
                signer: ctx.accounts.stake.signer.key(),
                d_total_stake_num: user_stake_info.d_total_stake_num,
                stake_amount: delegate_info.stake_amount,
                pool_creator: ctx.accounts.pool_creator.to_account_info().key(),
            }
        );
		Ok(())
	}
	
	pub fn delegator_unstake(ctx : Context<DelegatorUnStake>, unstake_amount: u64) -> Result<()> {
		let user_stake_info		= &mut ctx.accounts.unstake.user_stake_info;
		let delegate_info		= &mut ctx.accounts.delegate_info;
		let all_config			= ctx.accounts.unstake.all_config.clone();
		
		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.unstake.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

		require!(
			unstake_amount <= delegate_info.stake_amount,
			ErrorCode::UnStakeToMuch
		);

		user_stake_info.d_total_stake_num	-= unstake_amount;
		delegate_info.stake_amount			-= unstake_amount;
		
		if delegate_info.stake_amount == 0{
			ctx.accounts.verifier_pool.delegator_num -= 1;
		}

		let clock = Clock::get().unwrap();
        let now_time = match ctx.accounts.verifier_pool.is_working{
			true => clock.unix_timestamp,
			false => ctx.accounts.verifier_pool.stop_time,
		}; 
		
		if delegate_info.unstake_wait_time <= now_time{
			delegate_info.wait_claim_unstake	+= delegate_info.ongoing_unstake;
			delegate_info.ongoing_unstake		= 0;
		}
		
		delegate_info.unstake_wait_time 		= now_time + all_config.delegator_unstake_lock_time;
		delegate_info.ongoing_unstake			+= unstake_amount;

		crate::log!("delegator_unstake", ctx.accounts.unstake.signer.key(), [user_stake_info.d_total_stake_num, delegate_info.stake_amount, delegate_info.ongoing_unstake, now_time, delegate_info.wait_claim_unstake, ctx.accounts.pool_creator.to_account_info().key()]);
	    emit!(
            DelegatorUnstakeEvent{
                signer: ctx.accounts.unstake.signer.key(),
                d_total_stake_num: user_stake_info.d_total_stake_num,
                stake_amount: delegate_info.stake_amount,
                ongoing_unstake: delegate_info.ongoing_unstake,
                now_time: now_time,
                wait_claim_unstake: delegate_info.wait_claim_unstake,
                pool_creator: ctx.accounts.pool_creator.to_account_info().key(),
            }
        );
        Ok(())
	}
	
	pub fn claim_worker_unstake(ctx : Context<ClaimUnStake>) -> Result<()> {
		let user_unstake_info	= &mut ctx.accounts.user_unstake_info;
		let all_config			= &mut ctx.accounts.all_config;
		
		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

        let now_time = public::now_time_s();

		if user_unstake_info.w_unstake_wait_time <= now_time{
			user_unstake_info.w_wait_claim_unstake	+= user_unstake_info.w_ongoing_unstake;
			user_unstake_info.w_ongoing_unstake		= 0;
		}

		require!(
			0 < user_unstake_info.w_wait_claim_unstake,
			ErrorCode::ClaimUnStakeToMuch
		);
		
		// transfer
        let all_config_seed_str = "all_config";

        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer = [&seeds[..]];

		let w_wait_claim_unstake	= user_unstake_info.w_wait_claim_unstake;

		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			w_wait_claim_unstake * all_config.token_decimals,
		)?;
		
		all_config.worker_stake_num				-= w_wait_claim_unstake;
		
		user_unstake_info.w_wait_claim_unstake	= 0;
		
		crate::log!("claim_worker_unstake", ctx.accounts.signer.key(), [user_unstake_info.w_ongoing_unstake]);
        emit!(
            ClaimWorkerUnstakeEvent{
                signer: ctx.accounts.signer.key(),
                w_ongoing_unstake: user_unstake_info.w_ongoing_unstake,
            }
        );
		Ok(())
	}
	
	pub fn claim_verifier_unstake(ctx : Context<ClaimUnStake>) -> Result<()> {
		let user_unstake_info	= &mut ctx.accounts.user_unstake_info;
		let all_config			= &mut ctx.accounts.all_config;
		
		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

        let now_time = public::now_time_s();

		if user_unstake_info.v_unstake_wait_time <= now_time{
			user_unstake_info.v_wait_claim_unstake	+= user_unstake_info.v_ongoing_unstake;
			user_unstake_info.v_ongoing_unstake		= 0;
		}

		require!(
			0 < user_unstake_info.v_wait_claim_unstake,
			ErrorCode::ClaimUnStakeToMuch
		);
		
		// transfer
        let all_config_seed_str = "all_config";

        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.all_config],
        ];
        let signer = [&seeds[..]];

		let v_wait_claim_unstake = user_unstake_info.v_wait_claim_unstake;

		public::transfer_to_with_signer(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.all_config_account.to_account_info(),
			ctx.accounts.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			v_wait_claim_unstake * all_config.token_decimals,
		)?;
		
		all_config.verifier_stake_num			-= v_wait_claim_unstake;
		
		user_unstake_info.v_wait_claim_unstake 	= 0;
		
		crate::log!("claim_verifier_unstake", ctx.accounts.signer.key(), [user_unstake_info.v_ongoing_unstake]);
		emit!(
            ClaimVerifierUnstakeEvent{
                signer: ctx.accounts.signer.key(),
                v_ongoing_unstake: user_unstake_info.v_ongoing_unstake,
            }
        );
		Ok(())
	}
	
	pub fn claim_delegator_unstake(ctx : Context<ClaimDelegateUnStake>) -> Result<()> {
		let all_config			= &mut ctx.accounts.claim_unstake.all_config;
		let delegate_info		= &mut ctx.accounts.delegate_info;
		
		require!(
            all_config.allow_unstake == true,
            ErrorCode::SystemDenyUnStake
        );
        
        require!(
            ctx.accounts.claim_unstake.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );

        let now_time = public::now_time_s();

		if delegate_info.unstake_wait_time <= now_time{
			delegate_info.wait_claim_unstake	+= delegate_info.ongoing_unstake;
			delegate_info.ongoing_unstake		= 0;
		}

		require!(
			0 < delegate_info.wait_claim_unstake,
			ErrorCode::ClaimUnStakeToMuch
		);
		
		// transfer
        let all_config_seed_str = "all_config";

        let seeds = &[
            all_config_seed_str.as_bytes(),
            &[ctx.bumps.claim_unstake.all_config],
        ];
        let signer = [&seeds[..]];

		let wait_claim_unstake	= delegate_info.wait_claim_unstake;

		public::transfer_to_with_signer(
			ctx.accounts.claim_unstake.token_program.to_account_info(),
			ctx.accounts.claim_unstake.all_config_account.to_account_info(),
			ctx.accounts.claim_unstake.signer_account.to_account_info(),
			all_config.to_account_info(),
			signer,
			wait_claim_unstake * all_config.token_decimals,
		)?;
		
		all_config.delegator_stake_num			-= wait_claim_unstake;
		
		delegate_info.wait_claim_unstake 	= 0;
		
		crate::log!("claim_delegator_unstake", ctx.accounts.claim_unstake.signer.key(), [delegate_info.ongoing_unstake, ctx.accounts.pool_creator.to_account_info().key()]);
		emit!(
            ClaimDelegatorUnstakeEvent{
                signer: ctx.accounts.claim_unstake.signer.key(),
                ongoing_unstake: delegate_info.ongoing_unstake,
                pool_creator: ctx.accounts.pool_creator.to_account_info().key(),
            }
        );
		Ok(())
	}
}

/******************************************************
    STRUCT
******************************************************/ 

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitSystemConfig {
    pub worker_unstake_lock_time : i64,
    pub verifier_unstake_lock_time: i64,
    pub delegator_unstake_lock_time: i64,
    
    pub worker_need_count       : u64,
    pub verifier_need_count_l1  : u64,
    pub verifier_need_count_l2  : u64,
    pub verifier_need_count_l3  : u64,
    pub verifier_need_count_l4  : u64,
    pub verifier_need_count_l5  : u64,
    pub allow_unstake            : bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VerifierStakeInfo {
	pub stake_amount			: u64,
	pub nonce					: u64,
	pub timestamp				: i64,
	pub signature				: [u8; 64],
}

/******************************************************
    CONTEXT
******************************************************/ 

#[derive(Accounts)]
#[instruction(
    params: InitSystemConfig
)]
pub struct Config<'info> {
    #[account(mut)]
    pub stake_token         : Account<'info, Mint>,

	#[account(init_if_needed, seeds = [b"all_config"], bump, payer = signer, space = 8 + 32 + 8 * 13 + 1)]
    pub all_config          : Account<'info, AllConfig>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint		= stake_token,
        associated_token::authority	= all_config,
    )]
    pub all_config_account  : Account<'info, TokenAccount>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub token_program       : Program<'info, Token>,
    pub system_program      : Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct UpdateBlackList<'info> {

    pub black_addr          : UncheckedAccount<'info>,

    #[account(init_if_needed, seeds = [b"black_list", black_addr.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     : Account<'info, BlackList>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub system_program      : Program<'info, System>,
}

#[derive(Accounts)]
pub struct SystemConfiscate<'info> {

    #[account(mut, address = all_config.stake_token)]
    pub stake_token         : Account<'info, Mint>,

    #[account(mut, seeds = [b"all_config"], bump)]
    pub all_config          : Account<'info, AllConfig>,

    #[account(
        mut, 
        associated_token::mint      = stake_token,
        associated_token::authority = all_config,
    )]
    pub all_config_account  : Account<'info, TokenAccount>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = stake_token,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
    
    pub token_program       : Program<'info, Token>,
    pub system_program      : Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Stake<'info> {

    #[account(mut, address = all_config.stake_token)]
    pub stake_token         : Account<'info, Mint>,

    #[account(mut, seeds = [b"all_config"], bump)]
    pub all_config          : Account<'info, AllConfig>,

    #[account(
        mut, 
        associated_token::mint      = stake_token,
        associated_token::authority = all_config,
    )]
    pub all_config_account  : Account<'info, TokenAccount>,

    #[account(init_if_needed, seeds = [b"black_list", signer.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     : Account<'info, BlackList>,

    #[account(
        init_if_needed,
        seeds = [b"stake_info", signer.key().as_ref()],
        bump,
        payer = signer,
        space = 8 + 8 + 8 + 1 + 4 + 8
    )]
    pub user_stake_info     : Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub signer              : Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint      = stake_token,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
    
    pub token_program       : Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
	
	pub system_program      : Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    params: VerifierStakeInfo
)]
pub struct VerifierStake<'info> {
    pub stake				: Stake<'info>,
	
	#[account(
		init_if_needed,
		seeds = [b"v_pool", stake.signer.key().as_ref()],
		bump,
		payer = stake.signer,
		space = 8 + 8 + 8 + 1
	)]
	pub verifier_pool		: Account<'info, VerifierPool>,
	
	#[account(
		init_if_needed, 
		seeds = [b"s_c", stake.signer.key().as_ref(), params.stake_amount.to_le_bytes().as_ref(), params.nonce.to_le_bytes().as_ref(), params.timestamp.to_le_bytes().as_ref()], 
		bump, 
		payer = stake.signer, 
		space = 8 + 1)]
    pub stake_check     			: Account<'info, StakeCheck>,
	
	#[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar	: AccountInfo<'info>,
	pub system_program      : Program<'info, System>,
}

#[derive(Accounts)]
pub struct DelegatorStake<'info> {
    pub stake				: Stake<'info>,
	
	pub pool_creator		: UncheckedAccount<'info>,
	
	#[account(
		mut,
		seeds = [b"v_pool", pool_creator.key().as_ref()],
		bump,
	)]
	pub verifier_pool		: Account<'info, VerifierPool>,
	
	#[account(
		init_if_needed,
		seeds = [b"d_i", stake.signer.key().as_ref(), verifier_pool.key().as_ref()],
		bump,
		payer = stake.signer,
		space = 8 + 8 + 8 + 8 + 8
	)]
	pub delegate_info		: Account<'info, DelegateInfo>,
	
	#[account(mut)]
    pub signer              : Signer<'info>,
	pub system_program      : Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnStake<'info> {

    #[account(mut, address = all_config.stake_token)]
    pub stake_token         : Account<'info, Mint>,

    #[account(seeds = [b"all_config"], bump)]
    pub all_config          : Account<'info, AllConfig>,

    #[account(init_if_needed, seeds = [b"black_list", signer.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     : Account<'info, BlackList>,

    #[account(mut, seeds = [b"stake_info", signer.key().as_ref()], bump)]
    pub user_stake_info     : Account<'info, UserStakeInfo>,

    #[account(init_if_needed, seeds = [b"unstake_info", signer.key().as_ref()], bump, payer = signer, space = 8 + 8 * 6)]
    pub user_unstake_info    : Account<'info, UserUnStakeInfo>,

    #[account(mut)]
    pub signer              : Signer<'info>,
    
    pub token_program       : Program<'info, Token>,
    pub system_program      : Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct VerifierUnStake<'info> {
	pub unstake				: UnStake<'info>,
	pub pool_creator		: UncheckedAccount<'info>,
	
	#[account(
		mut,
		seeds = [b"v_pool", pool_creator.key().as_ref()],
		bump,
	)]
	pub verifier_pool		: Account<'info, VerifierPool>,
}

#[derive(Accounts)]
pub struct DelegatorUnStake<'info> {
	pub unstake				: UnStake<'info>,
	pub pool_creator		: UncheckedAccount<'info>,
	
	#[account(
		mut,
		seeds = [b"v_pool", pool_creator.key().as_ref()],
		bump,
	)]
	pub verifier_pool		: Account<'info, VerifierPool>,
	
	#[account(
		mut,
		seeds = [b"d_i", unstake.signer.key().as_ref(), verifier_pool.key().as_ref()],
		bump,
	)]
	pub delegate_info		: Account<'info, DelegateInfo>,
}

#[derive(Accounts)]
pub struct ClaimUnStake<'info> {

    #[account(mut, address = all_config.stake_token)]
    pub stake_token         : Account<'info, Mint>,

    #[account(mut, seeds = [b"all_config"], bump)]
    pub all_config          : Account<'info, AllConfig>,

	#[account(
        mut, 
        associated_token::mint      = stake_token,
        associated_token::authority = all_config,
    )]
    pub all_config_account  : Account<'info, TokenAccount>,

    #[account(init_if_needed, seeds = [b"black_list", signer.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     : Account<'info, BlackList>,

    #[account(init_if_needed, seeds = [b"unstake_info", signer.key().as_ref()], bump, payer = signer, space = 8 + 8 * 6)]
    pub user_unstake_info    : Account<'info, UserUnStakeInfo>,

    #[account(mut)]
    pub signer              : Signer<'info>,
    
	#[account(
        mut,
        associated_token::mint      = stake_token,
        associated_token::authority = signer,
    )]
    pub signer_account      : Account<'info, TokenAccount>,
	
    pub token_program       : Program<'info, Token>,
    pub system_program      : Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ClaimDelegateUnStake<'info> {
	pub claim_unstake		: ClaimUnStake<'info>,
	pub pool_creator		: UncheckedAccount<'info>,
	
	#[account(
		mut,
		seeds = [b"v_pool", pool_creator.key().as_ref()],
		bump,
	)]
	pub verifier_pool		: Account<'info, VerifierPool>,
	
	#[account(
		mut,
		seeds = [b"d_i", claim_unstake.signer.key().as_ref(), verifier_pool.key().as_ref()],
		bump,
	)]
	pub delegate_info		: Account<'info, DelegateInfo>,
}

/******************************************************
    ACCOUNT
******************************************************/ 

#[account]
pub struct AllConfig {
    pub stake_token                 : Pubkey,
    pub token_decimals              : u64,

    pub worker_stake_num            : u64,
    pub verifier_stake_num          : u64,
	pub delegator_stake_num			: u64,

    pub worker_unstake_lock_time     : i64,
    pub verifier_unstake_lock_time   : i64,
    pub delegator_unstake_lock_time  : i64,
    
    pub worker_need_count           : u64,
    pub verifier_need_count_l1      : u64,
    pub verifier_need_count_l2      : u64,
    pub verifier_need_count_l3      : u64,
    pub verifier_need_count_l4      : u64,
    pub verifier_need_count_l5      : u64,

    pub allow_unstake                : bool,
}

#[account]
pub struct BlackList {
    pub need_block                  : bool,
}

#[account]
pub struct UserStakeInfo {
    pub w_total_stake_num			: u64,
    pub w_allow_online_device_num	: u32,
    pub v_total_stake_num			: u64,
    pub v_allow_online_device_level : u8,
	pub d_total_stake_num			: u64,
}

#[account]
pub struct VerifierPool {
	pub delegator_num				: u64,
	pub is_working					: bool,
	pub stop_time					: i64,
}

#[account]
pub struct StakeCheck {
    pub has_been_used			: bool,
}

#[account]
pub struct DelegateInfo {
	pub stake_amount				: u64,
	pub ongoing_unstake				: u64,
	pub wait_claim_unstake			: u64,
	pub unstake_wait_time			: i64,
}

#[account]
pub struct UserUnStakeInfo {
    pub w_wait_claim_unstake		: u64,
    pub w_ongoing_unstake			: u64,
    pub w_unstake_wait_time			: i64,
    pub v_wait_claim_unstake		: u64,
    pub v_ongoing_unstake			: u64,
    pub v_unstake_wait_time			: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("This user has been blocked.")]
    UserHasBeenBlock,
    #[msg("The system cannot allow unstake now.")]
    SystemDenyUnStake,
	#[msg("Your unstake amount must be less than your stake.")]
    UnStakeToMuch,
	#[msg("Your claim unstake amount must be less than your claimable amount.")]
    ClaimUnStakeToMuch,
	#[msg("Need backend sig.")]
	SignatureMissing,
	#[msg("Signature lenght error.")]
	SignatureDataLenErr,
	#[msg("No backend sig.")]
	SignatureOwnerError,
	#[msg("Signature data error.")]
	SignatureDataErr,
	#[msg("This stake signature has been used.")]
	StakeSigAlreadyUse,
	#[msg("This pool has been stopped.")]
	VerifierPoolStop,
}



#[event]
pub struct WorkerStakeEvent {
   pub signer: Pubkey,
   pub w_total_stake_num: u64,
}
#[event]
pub struct WorkerUnstakeEvent {
    pub signer: Pubkey,
    pub w_total_stake_num: u64,
    pub w_ongoing_unstake: u64,
    pub now_time: i64,
    pub w_wait_claim_unstake: u64,
}

#[event]
pub struct VerifierStakeEvent {
    pub signer: Pubkey,
    pub v_total_stake_num: u64,
    pub nonce: u64,
}

#[event]
pub struct VerifierUnstakeEvent {
    pub signer: Pubkey,
    pub v_total_stake_num: u64,
    pub v_ongoing_unstake: u64,
    pub now_time: i64,
    pub v_wait_claim_unstake: u64,
}

#[event]
pub struct DelegatorStakeEvent {
    pub signer: Pubkey,
    pub d_total_stake_num: u64,
    pub stake_amount: u64,
    pub pool_creator: Pubkey,
}

#[event]
pub struct DelegatorUnstakeEvent {
    pub signer: Pubkey,
    pub d_total_stake_num: u64,
    pub stake_amount: u64,
    pub ongoing_unstake: u64,
    pub now_time: i64,
    pub wait_claim_unstake: u64,
    pub pool_creator: Pubkey,
}

#[event]
pub struct ClaimWorkerUnstakeEvent {
    pub signer: Pubkey,
    pub w_ongoing_unstake: u64,
}


#[event]
pub struct ClaimVerifierUnstakeEvent {
    pub signer: Pubkey,
    pub v_ongoing_unstake: u64,
}

#[event]
pub struct ClaimDelegatorUnstakeEvent {
    pub signer: Pubkey,
    pub ongoing_unstake: u64,
    pub pool_creator: Pubkey,
}