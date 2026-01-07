import * as anchor from '@coral-xyz/anchor';
import type { Program } from '@coral-xyz/anchor';
import { AuthorityType, TOKEN_2022_PROGRAM_ID, createAccount, createMint, getAccount, mintTo, setAuthority } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import type { ImmutableOwner } from '../target/types/immutable_owner';

describe('immutable-owner', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.ImmutableOwner as Program<ImmutableOwner>;

  const tokenKeypair = new anchor.web3.Keypair();

  // it('Create Token Account with ImmutableOwner extension', async () => {
  //   const mint = await createMint(
  //     connection,
  //     wallet.payer, // Payer of the transaction and initialization fees
  //     wallet.publicKey, // Mint Authority
  //     null, // Optional Freeze Authority
  //     2, // Decimals of Mint
  //     undefined, // Optional keypair
  //     undefined, // Options for confirming the transaction
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   const transactionSignature = await program.methods
  //     .initialize()
  //     .accounts({
  //       mintAccount: mint,
  //       tokenAccount: tokenKeypair.publicKey,
  //     })
  //     .signers([tokenKeypair])
  //     .rpc({ skipPreflight: true });
  //   console.log('Your transaction signature', transactionSignature);
  // });

  // it('Attempt to change token account owner, expect fail', async () => {
  //   try {
  //     await setAuthority(
  //       connection, // Connection to use
  //       wallet.payer, // Payer of the transaction fee
  //       tokenKeypair.publicKey, // Token Account
  //       wallet.publicKey, // Owner of the Token Account
  //       AuthorityType.AccountOwner, // Type of Authority
  //       new anchor.web3.Keypair().publicKey, // Random address as new account Owner
  //       undefined, // Additional signers
  //       undefined, // Confirmation options
  //       TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //     );
  //   } catch (error) {
  //     console.log('\nExpect Error:', error.logs);
  //   }
  // });


  
  // it('Change token account owner', async () => {
  //   const mint = await createMint(
  //     connection,
  //     wallet.payer, // Payer of the transaction and initialization fees
  //     wallet.publicKey, // Mint Authority
  //     null, // Optional Freeze Authority
  //     0, // Decimals of Mint
  //     undefined, // Optional keypair
  //     undefined, // Options for confirming the transaction
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   const tokenAccount = await createAccount(
  //     connection,
  //     wallet.payer, // Payer to create Token Account
  //     mint, // Mint Account address
  //     wallet.payer.publicKey, // Token Account owner
  //     new anchor.web3.Keypair(), // Optional keypair
  //     undefined, // Confirmation options
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );
  //   const newOwner = new anchor.web3.Keypair();

  //   const transactionSignature = await program.methods
  //     .changeTokenOwner()
  //     .accounts({
  //       tokenAccount: tokenAccount,
  //       currentOwner: wallet.publicKey,
  //       newOwner: newOwner.publicKey,
  //     })
  //     .rpc({ skipPreflight: true });
  //   console.log('Your transaction signature', transactionSignature);
  
  // });

  it('Transfer PDA control', async () => {
    const newAuthority = new anchor.web3.Keypair();
    let [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pda_account_a")],
      program.programId
    );

    // let pdaAccount1 = await program.methods
    //   .initializePda()
    //   .accounts({
    //     authority: wallet.publicKey,
    //     pdaAccount:pda,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //   })
    //   .rpc();
    // console.log('PDA Account created:', pdaAccount1);


    const pdaAccountInfo = await connection.getAccountInfo(pda);
    console.log('PDA Data:', pdaAccountInfo?.data);
    const pdaAccountbefore = await program.account.pdaAccount.fetch(pda);
    console.log('New PDA Authority:', pdaAccountbefore.authority.toString());

    const transactionSignature = await program.methods
      .transferPdaControl(newAuthority.publicKey)
      .accounts({
        pdaAccount: pda,
        authority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log('Your transaction signature', transactionSignature);
    // Verify the new authority
    const pdaAccount = await program.account.pdaAccount.fetch(pda);
    console.log('New PDA Authority:', pdaAccount.authority.toString());
  });

  // it('Transfer NFT', async () => {

  //   const mint = await createMint(
  //     connection,
  //     wallet.payer, // Payer of the transaction and initialization fees
  //     wallet.publicKey, // Mint Authority
  //     null, // Optional Freeze Authority
  //     0, // Decimals of Mint
  //     undefined, // Optional keypair
  //     undefined, // Options for confirming the transaction
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   const tokenAccount = await createAccount(
  //     connection,
  //     wallet.payer, // Payer to create Token Account
  //     mint, // Mint Account address
  //     wallet.payer.publicKey, // Token Account owner
  //     new anchor.web3.Keypair(), // Optional keypair
  //     undefined, // Confirmation options
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   const newOwner = await createAccount(
  //     connection,
  //     wallet.payer, // Payer to create Token Account
  //     mint, // Mint Account address
  //     wallet.payer.publicKey, // Token Account owner
  //     new anchor.web3.Keypair(), // Optional keypair
  //     undefined, // Confirmation options
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   await mintTo(
  //     connection,
  //     wallet.payer, // Transaction fee payer
  //     mint, // Mint
  //     tokenAccount, // Mint to
  //     wallet.payer, // Mint authority
  //     1, // Amount
  //     [], // Additional signers
  //     null, // Commitment
  //     TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  //   );

  //   const transactionSignature = await program.methods
  //     .transferNft()
  //     .accounts({
  //       nftAccount: tokenAccount,
  //       newOwnerTokenAccount: newOwner,
  //       currentOwner: wallet.publicKey,
  //       mintAccount: mint,
  //       tokenProgram: TOKEN_2022_PROGRAM_ID,
  //     })
  //     .rpc({ skipPreflight: true });
  //   console.log('Your transaction signature', transactionSignature);
  // });

});