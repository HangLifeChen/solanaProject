import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenExample } from "../target/types/token_example";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddress,
  getMint,
  getAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
 
describe("token-example", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
 
  const program = anchor.workspace.TokenExample as Program<TokenExample>;
  const [mint, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId,
  );
 
  const [token, tokenBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("token")],
    program.programId,
  );
 
  it("Is initialized!", async () => {
    // const tx = await program.methods
    //   .createMint()
    //   .accounts({
    //     tokenProgram: TOKEN_2022_PROGRAM_ID,
    //   })
    //   .rpc({ commitment: "confirmed" });
    // console.log("Your transaction signature", tx);
 
    const mintAccount = await getMint(
      program.provider.connection,
      mint,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
 
    console.log("Mint Account", mintAccount);
  });
 
  it("Mint Tokens", async () => {
    const [ata] = await anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .mintTokens(new anchor.BN(100))
      .accounts({
        signer: program.provider.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        tokenAccount: ata,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        mint: mint,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .rpc({ commitment: "confirmed" });

    console.log("Your transaction signature", tx);
 
    const associatedTokenAccount = await getAssociatedTokenAddress(
      mint,
      program.provider.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
 
    const tokenAccount = await getAccount(
      program.provider.connection,
      associatedTokenAccount,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
 
    console.log("Token Account", tokenAccount);

  });
});