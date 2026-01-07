import * as anchor from '@coral-xyz/anchor';
import { BN, type Program } from '@coral-xyz/anchor';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, getAccount, getOrCreateAssociatedTokenAccount, mintTo, transfer } from '@solana/spl-token';
import type { NonTransferable } from '../target/types/non_transferable';

describe('non-transferable', () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.NonTransferable as Program<NonTransferable>;

  const mintKeypair = new anchor.web3.Keypair();
  const recipient = new anchor.web3.Keypair();

  it('Create Mint with NonTransferable extension', async () => {
    const transactionSignature = await program.methods
      .initialize()
      .accounts({ mintAccount: mintKeypair.publicKey })
      .signers([mintKeypair])
      .rpc({ skipPreflight: true });
    console.log('Your transaction signature', transactionSignature);
  });

  it('Attempt Token Transfer', async () => {
    const amount = 100;

    const sourceTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer, // Transaction fee payer
      mintKeypair.publicKey, // Mint
      wallet.publicKey, // Owner
      false, // Allow owner off curve
      null, // Commitment
      null, // Confirm options
      TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
      ASSOCIATED_TOKEN_PROGRAM_ID, // Associated Token Program ID
    );

    const destinationTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer, // Transaction fee payer
      mintKeypair.publicKey, // Mint
      recipient.publicKey, // Owner
      false, // Allow owner off curve
      null, // Commitment
      null, // Confirm options
      TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
      ASSOCIATED_TOKEN_PROGRAM_ID, // Associated Token Program ID
    );

    await mintTo(
      connection,
      wallet.payer, // Transaction fee payer
      mintKeypair.publicKey, // Mint
      sourceTokenAccount.address, // Mint to
      wallet.payer, // Mint authority
      amount, // Amount
      [], // Additional signers
      null, // Commitment
      TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
    );

     const tokenAccount = await getAccount(
      program.provider.connection,
      sourceTokenAccount.address,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );

    console.log('\nToken Account:', tokenAccount);
 

    try {
      // Attempt to Transfer tokens, expect error
      await transfer(
        connection,
        wallet.payer, // Transaction fee payer
        sourceTokenAccount.address, // Transfer from
        destinationTokenAccount.address, // Transfer to
        wallet.publicKey, // Source Token Account owner
        amount, // Amount
        undefined, // Additional signers
        undefined, // Confirmation options
        TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
      );
    } catch (error) {
      console.log('\nExpect Error:', error.logs);
    }

    const transactionSignature=await program.methods
      .processBurn(new BN(50))
      .accounts({
        owner: wallet.publicKey,
        mint: mintKeypair.publicKey,
        tokenAccount: sourceTokenAccount.address,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([wallet.payer])
      .rpc({ skipPreflight: true });
    console.log('Your transaction signature', transactionSignature);
    const sourceTokenAccount2=await getAccount(
      program.provider.connection,
      sourceTokenAccount.address,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
    console.log('Source Token Account 2:', sourceTokenAccount2);

  });
});