import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CurdApp } from "../target/types/curd_app";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("curd_app", () => {
  // 设置 provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CurdApp as Program<CurdApp>;
  const owner = provider.wallet;

  // 测试数据
    const title = `TestTitle-${Date.now()}`;
  const message = "Hello, this is my first journal entry!";
  const updatedMessage = "This is my updated journal entry.";

  // PDA
  let journalEntryPda: PublicKey;
  let bump: number;

  it("Initialize PDA address", async () => {
    [journalEntryPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from(title), owner.publicKey.toBuffer()],
      program.programId
    );
    console.log("PDA Address:", journalEntryPda.toBase58());
  });

  it("Create a journal entry", async () => {
    await program.methods
      .createJournalEntry(title, message)
      .accounts({
        owner: owner.publicKey,
      })
      .rpc();

    const account = await program.account.journalEntryState.fetch(
      journalEntryPda
    );

    assert.equal(account.owner.toBase58(), owner.publicKey.toBase58());
    assert.equal(account.title, title);
    assert.equal(account.content, message);
  });

  it("Update a journal entry", async () => {
    await program.methods
      .updateJournalEntry(title, updatedMessage)
      .accounts({
        journalEntry: journalEntryPda,
        owner: owner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.journalEntryState.fetch(
      journalEntryPda
    );

    assert.equal(account.content, updatedMessage);
  });

  it("Delete a journal entry", async () => {
    await program.methods
      .deleteJournalEntry(title)
      .accounts({
        journalEntry: journalEntryPda,
        owner: owner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    try {
      await program.account.journalEntryState.fetch(journalEntryPda);
      assert.fail("Account should be closed but still exists.");
    } catch (err) {
      assert.include(err.toString(), "Account does not exist");
    }
  });
});
