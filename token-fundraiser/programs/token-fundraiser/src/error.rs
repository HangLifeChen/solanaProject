use anchor_lang::prelude::*;

#[error_code]
pub enum  FundraiserError {
   #[msg("The amount to raise has not been met")]
   TargetNotMet,
   #[msg("The amount to raise has been achieved")]
   TargetMet,
   #[msg("The contribution is too big")]
   ContributionTooBig,
   #[msg("The contribution is too small")]
   ContributionTooSmall,
   #[msg("The maximum amount to contribute has been reached")]
   MaxContributionReached,
   #[msg("The fundraiser has not ended yet")]
   FundraiserNotEnded,
   #[msg("The fundraiser has already ended")]
   FundraiserEnded,
   #[msg("Invalid total amount. It should be bigger than 3")]
   InvalidAmount,
}