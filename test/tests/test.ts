import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SwapExample } from "../target/types/swap_example";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { type Connection, Keypair, PublicKey, type Signer } from '@solana/web3.js';
import { BN } from 'bn.js';
import { expect } from "chai";

const provider = anchor.AnchorProvider.env();
const creator = provider.wallet;
const connection = provider.connection;
const mintedAmount = 100;
const decimals = 6;
const holder = creator;
const id =  Keypair.generate().publicKey;
const mintAKeypair = Keypair.generate();
const mintBKeypair = Keypair.generate();
const depositAmountA= new BN(4 * 10 ** 6);
const depositAmountB= new BN(1 * 10 ** 6);
const minimumLiquidity= new BN(100);
const defaultSupply= new BN(100 * 10 ** 6);

const admin = Keypair.generate();
const ammKey = PublicKey.findProgramAddressSync([id.toBuffer()], anchor.workspace.SwapExample.programId)[0];
const fee = 500; // Set the fee to 0.1% (in basis points)
const poolAuthority = PublicKey.findProgramAddressSync(
  [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer(), Buffer.from('authority')],
  anchor.workspace.SwapExample.programId,
)[0];
const mintLiquidity = PublicKey.findProgramAddressSync(
  [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer(), Buffer.from('liquidity')],
  anchor.workspace.SwapExample.programId,
)[0];
const poolKey = PublicKey.findProgramAddressSync(
  [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer()],
  anchor.workspace.SwapExample.programId,
)[0];

const poolAccountA= getAssociatedTokenAddressSync(mintAKeypair.publicKey, poolAuthority, true,TOKEN_2022_PROGRAM_ID);
const poolAccountB= getAssociatedTokenAddressSync(mintBKeypair.publicKey, poolAuthority, true,TOKEN_2022_PROGRAM_ID);
const liquidityAccount= getAssociatedTokenAddressSync(mintLiquidity, creator.publicKey, true, TOKEN_2022_PROGRAM_ID);
const holderAccountA= getAssociatedTokenAddressSync(mintAKeypair.publicKey, creator.publicKey, true, TOKEN_2022_PROGRAM_ID);
const holderAccountB= getAssociatedTokenAddressSync(mintBKeypair.publicKey, creator.publicKey, true, TOKEN_2022_PROGRAM_ID);

describe("test", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.swap_example as Program<SwapExample>;
  

  it("Is initialized!", async () => {

  await program.methods.createAmm(id, fee).accounts({ amm: ammKey, admin: admin.publicKey }).rpc();
  console.log("Minting tokens...");
  // Mint tokens
  let result=await connection.confirmTransaction(await connection.requestAirdrop(creator.publicKey, 10 ** 10));
  console.log("Airdrop result:",result);
  await createMint(connection, creator.payer, creator.publicKey, creator.publicKey, decimals, mintAKeypair,null,TOKEN_2022_PROGRAM_ID);
  await createMint(connection, creator.payer, creator.publicKey, creator.publicKey, decimals, mintBKeypair,null,TOKEN_2022_PROGRAM_ID);
  console.log("Created mints.");
  await getOrCreateAssociatedTokenAccount(connection, holder.payer, mintAKeypair.publicKey, holder.publicKey, true,null,null,TOKEN_2022_PROGRAM_ID);
  await getOrCreateAssociatedTokenAccount(connection, holder.payer, mintBKeypair.publicKey, holder.publicKey, true,null,null,TOKEN_2022_PROGRAM_ID);
  console.log("Minted tokens.");
  await mintTo(
    connection,
    creator.payer,
    mintAKeypair.publicKey,
    getAssociatedTokenAddressSync(mintAKeypair.publicKey, holder.publicKey, true,TOKEN_2022_PROGRAM_ID),
    creator.publicKey,
    mintedAmount * 10 ** decimals,
    [], // Additional signers
    null, // Commitment
    TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  );
  await mintTo(
    connection,
    creator.payer,
    mintBKeypair.publicKey,
    getAssociatedTokenAddressSync(mintBKeypair.publicKey, holder.publicKey, true,TOKEN_2022_PROGRAM_ID),
    creator.publicKey,
    mintedAmount * 10 ** decimals,
    [], // Additional signers
    null, // Commitment
    TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  );
  console.log("Minted to holder.");
  });


  it("Creates a pool", async () => {
    await program.methods.createPool().accounts({
      pool: poolKey,
      amm: ammKey,
      mintA: mintAKeypair.publicKey,
      mintB: mintBKeypair.publicKey,
      poolAuthority: poolAuthority,
      mintLiquidity: mintLiquidity,
      payer: creator.publicKey,
      poolAccountA: poolAccountA,
      poolAccountB: poolAccountB,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
    }).rpc();
  });

  it('Deposit equal amounts', async () => {
    await program.methods
      .depositLiquidity(depositAmountA, depositAmountB)
      .accounts({
        pool: poolKey,
        poolAuthority: poolAuthority,
        depositor: creator.publicKey,
        mintLiquidity: mintLiquidity,
        mintA: mintAKeypair.publicKey,
        mintB: mintBKeypair.publicKey,
        poolAccountA: poolAccountA,
        // poolAccountB: poolAccountB,
        // depositLiquidityAccount: liquidityAccount,
        // depositAmountA: depositAmountA,
        // depositAmountB: depositAmountB,
        payer: creator.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc({ skipPreflight: true });

    // const depositTokenAccountLiquditiy = await connection.getTokenAccountBalance(liquidityAccount);
    // expect(depositTokenAccountLiquditiy.value.amount).to.equal(depositAmountA.sub(minimumLiquidity).toString());
    // const depositTokenAccountA = await connection.getTokenAccountBalance(holderAccountA);
    // expect(depositTokenAccountA.value.amount).to.equal(defaultSupply.sub(depositAmountA).toString());
    // const depositTokenAccountB = await connection.getTokenAccountBalance(holderAccountB);
    // expect(depositTokenAccountB.value.amount).to.equal(defaultSupply.sub(depositAmountA).toString());
  });

});

