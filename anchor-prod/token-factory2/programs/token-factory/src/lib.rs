use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use solana_program::{
    ed25519_program,
    sysvar::{instructions::load_instruction_at_checked},
};
use std::str::FromStr;

mod public;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("CNAc7rGJi6qDBXwz3eMePkSobeBkByFdZdmn2jJnGJ3b");
const SYSTEM_OWNER: &str = "nVyjcxVWTddPkUF74VbrfByNKyGf2m3Kv4fmyQJNiyi";
const SYSTEM_PUBKEY: [u8; 32] = [
   11, 167, 230, 243,  52,  58, 151, 249,
   63, 219, 156, 241,  19, 112,   4, 105,
   49,  50,  29,  85, 166, 126, 184,  27,
  223,  23,  62,  34, 138,  60, 160,  85
];


#[program]
mod token_factory{
    use anchor_spl::token::spl_token;

    use super::*;

	const DECIMALS: u64 = 1000000;

    pub fn create_token(ctx: Context<CreateToken>, params: InitTokenParams) -> Result<()> {
	
        let random_num1_	= params.random_num1;
        let random_num2_	= params.random_num2;

        let rs_s_			= params.random_str;
        let rn1_s_			= random_num1_.to_le_bytes();
        let rn2_s_			= random_num2_.to_le_bytes();

        let seeds = &[
            rs_s_.as_bytes(),
            rn1_s_.as_ref(),
            rn2_s_.as_ref(),
            &[ctx.bumps.mint],
        ];
        let signer	= [&seeds[..]];

        let name_	= params.name.clone();
        let symbol_	= params.symbol.clone();
        let uri_	= params.uri.clone();

        let token_data: DataV2 = DataV2 {
            name: name_,
            symbol: symbol_,
            uri: uri_,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );

        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;
        
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.signer_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            params.total_suply * DECIMALS,
        )?;

		if params.give_up_auth{
			anchor_spl::token::set_authority(
				CpiContext::new_with_signer(
					ctx.accounts.token_program.to_account_info(),
					anchor_spl::token::SetAuthority {
						account_or_mint		: ctx.accounts.mint.to_account_info(),
						current_authority	: ctx.accounts.mint.to_account_info(),
					},
					&signer,
				),
				// spl_token::instruction::AuthorityType::AccountOwner,
				// Some(Pubkey::default()),
				spl_token::instruction::AuthorityType::MintTokens,
				None,
			)?;
		}
        Ok(())
    }
	
	pub fn config_system(ctx: Context<UpdateSystemConfig>, params: ConfigSystemParams) -> Result<()> {
        let all_config			= &mut ctx.accounts.all_config;
		
		all_config.pause_mint	= params.pause_mint;
		all_config.neb			= params.neb;
		all_config.es_neb		= params.es_neb;
		
        Ok(())
    }
	
	pub fn config_black_list(ctx: Context<UpdateBlackList>, new_config: bool) -> Result<()> {
        ctx.accounts.user_black_info.need_block = new_config;
        Ok(())
    }
	
	pub fn mint_token(ctx: Context<MintToken>, params: MintTokenParams) -> Result<()> {
		
		let mint_check			= &mut ctx.accounts.mint_check;
		
		require!(mint_check.has_been_used == false, ErrorCode::MintSigAlreadyUse);
		
		mint_check.has_been_used= true;
		
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
		expected_message.extend_from_slice(ctx.accounts.signer.key.as_ref());
		expected_message.extend_from_slice(&params.amount.to_le_bytes());
		expected_message.extend_from_slice(&params.nonce.to_le_bytes());
		expected_message.extend_from_slice(&params.timestamp.to_le_bytes());
		
        if message_bytes != expected_message.as_slice() {
            return err!(ErrorCode::SignatureDataErr);
        }

		let all_config			= &mut ctx.accounts.all_config;
		
		require!(
            all_config.pause_mint == false,
            ErrorCode::SystemDenyMint
        );
        
        require!(
            ctx.accounts.user_black_info.need_block == false,
            ErrorCode::UserHasBeenBlock
        );
		
		let random_num1_	= params.random_num1;
        let random_num2_	= params.random_num2;

        let rs_s_			= params.random_str;
        let rn1_s_			= random_num1_.to_le_bytes();
        let rn2_s_			= random_num2_.to_le_bytes();

        let seeds = &[
            rs_s_.as_bytes(),
            rn1_s_.as_ref(),
            rn2_s_.as_ref(),
            &[ctx.bumps.es_neb],
        ];
        let signer	= [&seeds[..]];
		
		mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.es_neb.to_account_info(),
                    to: ctx.accounts.signer_token_account.to_account_info(),
                    mint: ctx.accounts.es_neb.to_account_info(),
                },
                &signer,
            ),
            params.amount * DECIMALS,
        )?;
		
		crate::log!("mint_token", ctx.accounts.signer.key(), [params.amount, params.nonce]);
		
		Ok(())
	}
	
	pub fn change_neb_to_es_neb(ctx: Context<ChangeNeb2esNeb>, params: ChangeNeb2esNebParams) -> Result<()> {
		
		public::transfer_to(
			ctx.accounts.token_program.to_account_info(),
			ctx.accounts.signer_neb_account.to_account_info(),
			ctx.accounts.get_neb_account.to_account_info(),
			ctx.accounts.signer.to_account_info(),
			params.amount * DECIMALS,
		)?;
		
		let random_num1_	= params.random_num1;
        let random_num2_	= params.random_num2;

        let rs_s_			= params.random_str;
        let rn1_s_			= random_num1_.to_le_bytes();
        let rn2_s_			= random_num2_.to_le_bytes();

        let seeds = &[
            rs_s_.as_bytes(),
            rn1_s_.as_ref(),
            rn2_s_.as_ref(),
            &[ctx.bumps.es_neb],
        ];
        let signer	= [&seeds[..]];
		
		mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.es_neb.to_account_info(),
                    to: ctx.accounts.signer_es_neb_account.to_account_info(),
                    mint: ctx.accounts.es_neb.to_account_info(),
                },
                &signer,
            ),
            params.amount * DECIMALS,
        )?;
		
		crate::log!("change_neb_to_es_neb", ctx.accounts.signer.key(), [params.amount]);
		
		Ok(())
	}
}

/******************************************************
    STRUCT
******************************************************/ 

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name		: String,
    pub symbol		: String,
    pub uri			: String,
	pub total_suply	: u64,
	pub give_up_auth: bool,
	pub random_str  : String,
    pub random_num1 : u64,
    pub random_num2 : u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ConfigSystemParams {
	pub pause_mint	: bool,
	pub neb			: Pubkey,
	pub es_neb		: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MintTokenParams {
    pub amount		: u64,
	pub nonce		: u64,
	pub timestamp	: i64,
	pub signature	: [u8; 64],
	pub random_str  : String,
    pub random_num1 : u64,
    pub random_num2 : u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ChangeNeb2esNebParams {
	pub amount		: u64,
	pub random_str  : String,
    pub random_num1 : u64,
    pub random_num2 : u64,
}

/******************************************************
    CONTEXT
******************************************************/ 

#[derive(Accounts)]
#[instruction(
    params: InitTokenParams
)]
pub struct CreateToken<'info> {
    #[account(
        init,
        seeds = [params.random_str.as_bytes(), params.random_num1.to_le_bytes().as_ref(), params.random_num2.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint,
    )]
    pub mint					: Account<'info, Mint>,
    #[account(mut)]
    pub metadata				: UncheckedAccount<'info>,
    
    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer					: Signer<'info>,
	#[account(
        init,
        payer						= signer,
        associated_token::mint		= mint,
        associated_token::authority	= signer,
    )]
    pub signer_token_account	: Account<'info, TokenAccount>,
	
	
    pub rent					: Sysvar<'info, Rent>,
    pub system_program			: Program<'info, System>,
    pub token_program			: Program<'info, Token>,
    pub token_metadata_program	: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct UpdateSystemConfig<'info> {
	#[account(init_if_needed, seeds = [b"all_config"], bump, payer = signer, space = 8 + 1 + 32 + 32)]
    pub all_config          	: Account<'info, AllConfig>,

    #[account(mut, address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
    pub signer              : Signer<'info>,
    pub system_program      : Program<'info, System>,
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
#[instruction(
    params: MintTokenParams
)]
pub struct MintToken<'info> {

	#[account(mut, seeds = [b"all_config"], bump)]
    pub all_config          	: Account<'info, AllConfig>,

	#[account(init_if_needed, seeds = [b"black_list", signer.key().as_ref()], bump, payer = signer, space = 8 + 1)]
    pub user_black_info     	: Account<'info, BlackList>,

	#[account(
		mut,
		seeds = [params.random_str.as_bytes(), params.random_num1.to_le_bytes().as_ref(), params.random_num2.to_le_bytes().as_ref()],
		bump
	)]
    pub es_neb					: Account<'info, Mint>,

	#[account(
		init_if_needed, 
		seeds = [b"mint_check", signer.key().as_ref(), params.amount.to_le_bytes().as_ref(), params.nonce.to_le_bytes().as_ref(), params.timestamp.to_le_bytes().as_ref()], 
		bump, 
		payer = signer, 
		space = 8 + 1)]
    pub mint_check     			: Account<'info, MintCheck>,

	#[account(mut)]
    pub signer					: Signer<'info>,
	#[account(
        init_if_needed,
        payer 						= signer,
        associated_token::mint		= es_neb,
        associated_token::authority	= signer,
    )]
    pub signer_token_account	: Account<'info, TokenAccount>,
	pub system_program			: Program<'info, System>,
    pub token_program			: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
	#[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar		: AccountInfo<'info>,
}


#[derive(Accounts)]
#[instruction(
    params: ChangeNeb2esNebParams
)]
pub struct ChangeNeb2esNeb<'info> {
	
	#[account(seeds = [b"all_config"], bump)]
    pub all_config          	: Account<'info, AllConfig>,
	
	#[account(address = Pubkey::from_str(&SYSTEM_OWNER).unwrap())]
	pub system_owner: UncheckedAccount<'info>,
	
	#[account(
		init_if_needed,
        payer 						= signer,
        associated_token::mint		= neb,
        associated_token::authority	= system_owner
	)]
	pub get_neb_account			: Account<'info, TokenAccount>,
	
	#[account(mut, address = all_config.neb)]
    pub neb						: Account<'info, Mint>,
	
	#[account(
		mut,
		seeds = [params.random_str.as_bytes(), params.random_num1.to_le_bytes().as_ref(), params.random_num2.to_le_bytes().as_ref()],
		bump
	)]
    pub es_neb					: Account<'info, Mint>,
	
	#[account(
        init_if_needed,
        payer 						= signer,
        associated_token::mint		= neb,
        associated_token::authority	= signer,
    )]
    pub signer_neb_account		: Account<'info, TokenAccount>,
	
	#[account(
        init_if_needed,
        payer 						= signer,
        associated_token::mint		= es_neb,
        associated_token::authority	= signer,
    )]
    pub signer_es_neb_account	: Account<'info, TokenAccount>,
	
	#[account(mut)]
    pub signer					: Signer<'info>,
	pub system_program			: Program<'info, System>,
    pub token_program			: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/******************************************************
    ACCOUNT
******************************************************/ 

#[account]
pub struct AllConfig {
    pub pause_mint				: bool,
	pub es_neb					: Pubkey,
	pub neb						: Pubkey,
}

#[account]
pub struct BlackList {
    pub need_block				: bool,
}

#[account]
pub struct MintCheck {
    pub has_been_used			: bool,
}

#[error_code]
pub enum ErrorCode {
	#[msg("The system cannot allow mint now.")]
    SystemDenyMint,
    #[msg("This user has been blocked.")]
    UserHasBeenBlock,
	#[msg("Need backend sig.")]
	SignatureMissing,
	#[msg("Signature lenght error.")]
	SignatureDataLenErr,
	#[msg("No backend sig.")]
	SignatureOwnerError,
	#[msg("Signature data error.")]
	SignatureDataErr,
	#[msg("This mint signature has been used.")]
	MintSigAlreadyUse,
}