import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PermanentDelegate } from "../target/types/permanent_delegate";
import { createAssociatedTokenAccount, getAccount, mintTo, transfer, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("PermanentDelegate Test with spl-token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.PermanentDelegate as Program<PermanentDelegate>;

  const payer = provider.wallet;
  const mintKeypair = Keypair.generate();
  const userKeypair = Keypair.generate();
  const delegateKeypair = Keypair.generate();

  let mint: PublicKey;
  let userTokenAccount: PublicKey;
  let delegateTokenAccount: PublicKey;

  it("Initialize Mint with PermanentDelegate", async () => {
    mint = mintKeypair.publicKey;

    await program.methods
      .initialize()
      .accounts({
        payer: payer.publicKey,
        mintAccount: mint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([mintKeypair])
      .rpc();

    console.log("Mint initialized:", mint.toBase58());
  });

  it("Create user and delegate associated token accounts", async () => {
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      userKeypair.publicKey,
      null,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      false
    );

    delegateTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      delegateKeypair.publicKey,
      null,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      false
    );

    console.log("User ATA:", userTokenAccount.toBase58());
    console.log("Delegate ATA:", delegateTokenAccount.toBase58());
  });

  it("Mint tokens to user", async () => {
    await mintTo(
      provider.connection,
      payer.payer,
      mint,
      userTokenAccount,
      payer.payer, // mint authority
      100,
      [payer.payer],
      null,
      TOKEN_2022_PROGRAM_ID
    );

    const userAccountInfo = await getAccount(provider.connection, userTokenAccount,"confirmed",TOKEN_2022_PROGRAM_ID);
    assert.equal(Number(userAccountInfo.amount), 100);
    console.log("Minted 100 tokens to user");
  });

  it("Delegate burns user tokens", async () => {
    await program.methods
      .burnUserTokens(new anchor.BN(50))
      .accounts({
        delegate: payer.publicKey, // 使用 payer 作为 PermanentDelegate
        mint: mint,
        userTokenAccount: userTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const userAccountInfo = await getAccount(provider.connection, userTokenAccount,"confirmed",TOKEN_2022_PROGRAM_ID);
    assert.equal(Number(userAccountInfo.amount), 50);
    console.log("PermanentDelegate burned 50 tokens from user");
  });

  it("Delegate transfers user tokens", async () => {
    await program.methods
      .transferUserTokens(new anchor.BN(25))
      .accounts({
        delegate: payer.publicKey,
        mint: mint,
        from: userTokenAccount,
        to: delegateTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const userAccountInfo = await getAccount(provider.connection, userTokenAccount,"confirmed" ,TOKEN_2022_PROGRAM_ID);
    const delegateAccountInfo = await getAccount(provider.connection, delegateTokenAccount,"confirmed" ,TOKEN_2022_PROGRAM_ID);

    assert.equal(Number(userAccountInfo.amount), 25);
    assert.equal(Number(delegateAccountInfo.amount), 25);

    console.log("PermanentDelegate transferred 25 tokens from user to delegate");
  });
});
