use anchor_lang::{prelude::*, system_program::{create_account, CreateAccount}};
use anchor_spl::{
        token_2022::{
            initialize_mint2, 
            spl_token_2022::{
                extension::{
                transfer_fee::TransferFeeConfig,BaseStateWithExtensions,ExtensionType,
                StateWithExtensions
            },
                pod::PodMint,
                state::Mint as MintState,
            }, InitializeMint2
        },
        token_interface::{
            spl_pod::optional_keys::OptionalNonZeroPubkey,
            transfer_fee_initialize,
            Token2022,
            TransferFeeInitialize,
        }
    };


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn process_initialize(
    ctx: Context<Initialize>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<()> {
    let mint_size = ExtensionType::try_calculate_account_len::<PodMint>(&[ExtensionType::TransferFeeConfig])?;
    let lamports = (Rent::get()?).minimum_balance(mint_size);
    create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount{
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.mint_account.to_account_info(),
            },
        ),
        lamports,
        mint_size as u64,
        &ctx.accounts.token_program.key(),
    )?;

    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferFeeInitialize {
                mint: ctx.accounts.mint_account.to_account_info(),
                token_program_id: ctx.accounts.token_program.to_account_info(),
            },
        ),
        Some(&ctx.accounts.payer.key()),//手续费提取管理员
        Some(&ctx.accounts.payer.key()),//手续费配置管理员
        transfer_fee_basis_points, //手续费比例
        maximum_fee, //手续费上限
    )?;
   

    initialize_mint2(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            InitializeMint2 {
                mint: ctx.accounts.mint_account.to_account_info(),
            },
        ),
        2,
        &ctx.accounts.payer.key(),
        Some(&ctx.accounts.payer.key()),
    )?;

    
     Ok(())
}

impl<'info> Initialize<'info>{
    pub fn check_mint_data(&self) -> Result<()> {
        let mint = &self.mint_account.to_account_info();
        let mint_data = mint.data.borrow();
        let mint_with_extension = StateWithExtensions::<MintState>::unpack(&mint_data)?;
        let extension_data = mint_with_extension.get_extension::<TransferFeeConfig>()?;

        assert_eq!(
            extension_data.transfer_fee_config_authority,
            OptionalNonZeroPubkey::try_from(Some(self.payer.key()))?
        );
        
        assert_eq!(
            extension_data.transfer_fee_config_authority,
            OptionalNonZeroPubkey::try_from(Some(self.payer.key()))?
        );

        msg!("Mint data is valid {:?}", extension_data);
        Ok(())
    }

}