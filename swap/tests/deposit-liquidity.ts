import * as anchor from '@coral-xyz/anchor';
import type { Program } from '@coral-xyz/anchor';
import { expect } from 'chai';
import type { SwapExample } from '../target/types/swap_example';
import { type TestValues, createValues, mintingTokens } from './utils';
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';

describe('Deposit liquidity', () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(provider);

  const program = anchor.workspace.SwapExample as Program<SwapExample>;

  let values: TestValues;

  beforeEach(async () => {
    values = createValues();

    await program.methods.createAmm(values.id, values.fee).accounts({ amm: values.ammKey, admin: values.admin.publicKey }).rpc();

    await mintingTokens({
      connection,
      creator: values.admin,
      mintAKeypair: values.mintAKeypair,
      mintBKeypair: values.mintBKeypair,
    });

    await program.methods
      .createPool()
      .accounts({
        amm: values.ammKey,
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        mintLiquidity: values.mintLiquidity,
        mintA: values.mintAKeypair.publicKey,
        mintB: values.mintBKeypair.publicKey,
        poolAccountA: values.poolAccountA,
        poolAccountB: values.poolAccountB,
        payer: values.admin.publicKey,
        depositor: values.admin.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        ASSOCIATED_TOKEN_PROGRAM_ID: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

      console.log("Created pool.");
  });

  it('Deposit equal amounts', async () => {
    await program.methods
      .depositLiquidity(values.depositAmountA, values.depositAmountA)
      .accounts({
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        depositor: values.admin.publicKey,
        mintLiquidity: values.mintLiquidity,
        mintA: values.mintAKeypair.publicKey,
        mintB: values.mintBKeypair.publicKey,
        poolAccountA: values.poolAccountA,
        poolAccountB: values.poolAccountB,
        depositorAccountLiquidity: values.liquidityAccount,
        depositorAccountA: values.holderAccountA,
        depositorAccountB: values.holderAccountB,
        payer: values.admin.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        ASSOCIATED_TOKEN_PROGRAM_ID: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([values.admin])
      .rpc({ skipPreflight: true });

    const depositTokenAccountLiquditiy = await connection.getTokenAccountBalance(values.liquidityAccount);
    expect(depositTokenAccountLiquditiy.value.amount).to.equal(values.depositAmountA.sub(values.minimumLiquidity).toString());
    const depositTokenAccountA = await connection.getTokenAccountBalance(values.holderAccountA);
    expect(depositTokenAccountA.value.amount).to.equal(values.defaultSupply.sub(values.depositAmountA).toString());
    const depositTokenAccountB = await connection.getTokenAccountBalance(values.holderAccountB);
    expect(depositTokenAccountB.value.amount).to.equal(values.defaultSupply.sub(values.depositAmountA).toString());
  });
});
