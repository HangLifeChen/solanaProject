import * as anchor from '@coral-xyz/anchor';
import type { Program } from '@coral-xyz/anchor';
import { TOKEN_2022_PROGRAM_ID, amountToUiAmount } from '@solana/spl-token';
import type { InterestBearing } from '../target/types/interest_bearing';

describe('interest-bearing', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.InterestBearing as Program<InterestBearing>;

  const mintKeypair = new anchor.web3.Keypair();
  const rateState=new anchor.web3.Keypair();

  it('Create Mint with InterestBearingConfig extension', async () => {
    const rate = 0;

    const transactionSignature = await program.methods
      .initialize(rate)
      .accounts({ mintAccount: mintKeypair.publicKey })
      .signers([mintKeypair])
      .rpc({ skipPreflight: true });
    console.log('Your transaction signature', transactionSignature);
  });

  it('Update Mint with Interest Rate', async () => {
    const rate = 10;

    const transactionSignature = await program.methods.updateRate(rate).accounts({ mintAccount: mintKeypair.publicKey }).rpc({ skipPreflight: true });
    console.log('Your transaction signature', transactionSignature);
  });

  it('Calculate accrued interest', async () => {

    const createRateAccountTransactionSignature=await program.methods
      .createRateState()
      .accounts({
        payer: wallet.publicKey,
        rateState: rateState.publicKey
      })
      .signers([rateState])
      .rpc({ skipPreflight: true });
    console.log('Your transaction signature', createRateAccountTransactionSignature);
    let rateStateAccount = await program.account.rateState.fetch(rateState.publicKey);
    console.log('Rate State Account:', rateStateAccount);

    await sleep(1);
    const BN = require('bn.js');
    const slot = await connection.getSlot();
    // const solanaTimestamp = await connection.getBlockTime(slot);
    // console.log('Solana Timestamp:', solanaTimestamp);
    const date = new Date(2025, 6, 17, 12, 0, 0); // 注意月份是0-11
    const principal = new BN(1000); // example principal
    const startTime = new BN(Math.floor(date.getTime() / 1000));
    const transactionSignature = await program.methods
      .calculateInterest(principal, startTime)
      .accounts({
        mintAccount: mintKeypair.publicKey,
        rateState: rateState.publicKey
      })
      .rpc({ skipPreflight: true });
    console.log('Your transaction signature', transactionSignature);
    rateStateAccount = await program.account.rateState.fetch(rateState.publicKey);
    console.log('Rate State Account:', rateStateAccount);

    // const amount = 1000;
    // // Convert amount to UI amount with accrued interest
    // // This helper is a simulated transaction
    // const uiAmount = await amountToUiAmount(
    //   connection,
    //   wallet.payer,
    //   mintKeypair.publicKey, // Address of the Mint account
    //   amount, // Amount to be converted
    //   TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
    // );

    // console.log('\nAmount with Accrued Interest:', uiAmount);
  });
});

function sleep(s: number) {
  return new Promise((resolve) => setTimeout(resolve, s * 1000));
}