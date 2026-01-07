use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

pub fn transfer_to<'info>(token_program: AccountInfo<'info>, from: AccountInfo<'info>, to: AccountInfo<'info>, authority: AccountInfo<'info>, amount: u64) -> Result<()>{
    let cpi_accounts = anchor_spl::token::Transfer {
        from        : from,
        to          : to,
        authority   : authority,
    };

    let cpi_ctx = CpiContext::new(
        token_program,
        cpi_accounts,
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn transfer_to_with_signer<'info>(token_program: AccountInfo<'info>, from: AccountInfo<'info>, to: AccountInfo<'info>, authority: AccountInfo<'info>, signer: [&[&[u8]]; 1], amount: u64) -> Result<()>{
    let cpi_accounts = anchor_spl::token::Transfer {
        from        : from,
        to          : to,
        authority   : authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(
        token_program,
        cpi_accounts,
		&signer,
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn now_time_s() -> i64{
	let clock	= Clock::get().unwrap();
	clock.unix_timestamp
}

#[macro_export]
macro_rules! GET_ARRAY {
    ($account_info:expr, $type:ty, $len:expr) => {{
        let mut data_ref = ($account_info).try_borrow_mut_data()?;
        
        let bytes = &mut data_ref[8..];
        let array: &mut [$type; $len] = unsafe {
            &mut *(bytes.as_mut_ptr() as *mut [$type; $len])
        };
        array
    }};
}


#[macro_export]
macro_rules! log {
    ($method:expr, $signer:expr, [ $( $payload:expr ),* $(,)? ]) => {
        {
            let payloads = vec![$( format!("\"{}\"", $payload) ),*].join(",");
            let json = format!(
                r#"data|{{"method":"{}","signer":"{}","payload":[{}]}}"#,
                $method,
                $signer,
                payloads
            );
            msg!("{}", json);
        }
    };
}