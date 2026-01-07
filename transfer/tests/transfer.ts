import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TransferSol } from "../target/types/transfer_sol";

describe("transfer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.transferSol as Program<TransferSol>;
  const transferAmount = 10000;
 it("SOL Transfer Anchor", async () => {
  const transactionSignature = await program.methods
    .solTransfer(new anchor.BN(transferAmount))
    .accounts({
      sender: "8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1",
      recipient: "61tDUUStDX9f7prWab4VrLD9WxvEPHLZvus2LQEmNGUc",
      feeReceiver: "8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1",
    })
    .rpc();
 
    console.log(
      `\nTransaction Signature:` +
        `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`,
    );
  });


   it("SOL Transfer With Fee Anchor", async () => {
    const feeReceiver = new anchor.web3.PublicKey("8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1");
  const transactionSignature = await program.methods
    .solTransferWithFee(new anchor.BN(transferAmount))
    .accounts({
      sender: "8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1",
      recipient: "61tDUUStDX9f7prWab4VrLD9WxvEPHLZvus2LQEmNGUc",
      feeReceiver: feeReceiver,
    })
    .rpc();
 
    console.log(
      `\nTransaction Signature:` +
        `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`,
    );
  });
});
