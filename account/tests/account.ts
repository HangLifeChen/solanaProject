import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenExample } from "../target/types/token_example";
import { TOKEN_2022_PROGRAM_ID, getMint } from "@solana/spl-token";
 
describe("token-example", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
 
  const program = anchor.workspace.TokenExample as Program<TokenExample>;
  const [mint, bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId,
  );
 
  it("Is initialized!", async () => {
    const tx = await program.methods
      .createMint()
      .accounts({
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });
    console.log("Your transaction signature", tx);
 
    const mintAccount = await getMint(
      program.provider.connection,
      mint,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
 
    console.log("Mint Account", mintAccount);
  });
});