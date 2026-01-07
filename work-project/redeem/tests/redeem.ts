import * as anchor from "@coral-xyz/anchor";
import { BN, Program, web3 } from "@coral-xyz/anchor";
import { Redeem } from "../target/types/redeem";

// No imports needed: web3, anchor, pg and more are globally available
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.redeem as Program<Redeem>;
const wallet = provider.wallet as anchor.Wallet;

describe("Test", () => {
  // const contractPub = new web3.PublicKey(
  //   "8nzKvvpqJXLTxWdodvfZNTDfc8sJoCS8SSXAQT5ggiw5"
  // );

  const neb = new web3.PublicKey(
    "3tMkrR8M7eNiyfn2wFGGo5k1WDXAKubX6vth51vQAHeh"
  );

  const esNeb = new web3.PublicKey(
    "4k7iwv7CphLbRgU2UM9RprZgzhUBA3g6QPxdNqaj35th"
  );

  const tokenProgram = new web3.PublicKey(
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  );
  const associatedTokenProgram = new web3.PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
  );

  const [allConfig] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("all_config")],
     program.programId
  );

  const allConfigNebAccount = anchor.utils.token.associatedAddress({
    mint: neb,
    owner: allConfig,
  });

  const allConfigEsNebAccount = anchor.utils.token.associatedAddress({
    mint: esNeb,
    owner: allConfig,
  });

  console.log("all",allConfigEsNebAccount)

  const [userBlackInfo] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("black_list"), wallet.publicKey.toBuffer()],
     program.programId
  );

  const [allVaultInfo] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("all_v_info")],
     program.programId
  );

  const [rewardArray] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reward_array")],
     program.programId
  );

  it("config", async () => {

    let params = {
      allowRedeem: true,
      decimals: new BN(6),
    };

    try{
      const txHash = await program.methods
      .config(params)
      .accounts({
        allConfig: allConfig,
        esNeb: esNeb,
        neb: neb,
        signer: wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    }catch(err){
      console.log(err);
    }
  });

  // it("update_black_list", async () => {

  //   const need_block_this_user = false;

  //   const blackAddr = new web3.PublicKey("EhuFctMbCSQjZ1EHfZmAqZbnENouizVi8erFyNKaH4ay");

  //   try{
  //     const txHash = await pg.program.methods
  //     .updateBlackList(need_block_this_user)
  //     .accounts({
  //       blackAddr: blackAddr,
  //       userBlackInfo: userBlackInfo,
  //       signer: pg.wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //     })
  //     .rpc();
  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  //   }catch(err){
  //     console.log(err);
  //   }
  // });

  // it("config_stake", async () => {

  //   const stake_amount = new BN(100000);

  //   const allConfigNebAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: allConfig,
  //   });

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: wallet.publicKey,
  //   });

  //   try{
  //     const txHash = await program.methods
  //     .configStake(stake_amount)
  //     .accounts({
  //       neb: neb,
  //       allConfigNebAccount: allConfigNebAccount,
  //       allConfig: allConfig,
  //       signerAccount: signerAccount,
  //       signer: wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: tokenProgram,
  //       associatedTokenProgram: associatedTokenProgram,
  //     })
  //     .rpc();
  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  //   }catch(err){
  //     console.log(err);
  //   }
  // });

  // it("config_confiscate", async () => {

  //   const stake_amount = new BN(1000);

  //   const allConfigNebAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: allConfig,
  //   });

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: wallet.publicKey,
  //   });

  //   try{
  //     const txHash = await program.methods
  //     .configConfiscate()
  //     .accounts({
  //       neb: neb,
  //       allConfigNebAccount: allConfigNebAccount,
  //       allConfig: allConfig,
  //       signerAccount: signerAccount,
  //       signer: wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: tokenProgram,
  //       associatedTokenProgram: associatedTokenProgram,
  //     })
  //     .rpc();
  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  //   }catch(err){
  //     console.log(err);
  //   }
  // });

  // it("redeem", async () => {
  //   let param = {
  //     amount: new BN(10000),
  //     redeemType: 1,
  //   };

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: esNeb,
  //     owner: pg.wallet.publicKey,
  //   });

  //   const [userVaultInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_v_info"), pg.wallet.publicKey.toBuffer()],
  //     contractPub
  //   );

  //   const [userRedeemInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("redeem"), pg.wallet.publicKey.toBuffer()],
  //     contractPub
  //   );

  //   try {
  //     const txHash = await pg.program.methods
  //       .redeem(param)
  //       .accounts({
  //         allConfigEsNebAccount: allConfigEsNebAccount,
  //         allConfig: allConfig,
  //         esNeb: esNeb,

  //         vault: {
  //           rewardArray: rewardArray,
  //           userBlackInfo: userBlackInfo,
  //           allVaultInfo: allVaultInfo,
  //           userVaultInfo: userVaultInfo,
  //         },
  //         userRedeemInfo: userRedeemInfo,
  //         signerAccount: signerAccount,
  //         signer: pg.wallet.publicKey,
  //         systemProgram: web3.SystemProgram.programId,
  //         tokenProgram: tokenProgram,
  //         associatedTokenProgram: associatedTokenProgram,
  //       })
  //       .rpc();
  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  //   } catch (err) {
  //     console.log(err);
  //   }
  // });

  // it("redeem_claim", async () => {

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: pg.wallet.publicKey,
  //   });

  //   const [userRedeemInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("redeem"), pg.wallet.publicKey.toBuffer()],
  //     contractPub,
  //   );
  //   const [userVaultInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_v_info"), pg.wallet.publicKey.toBuffer()],
  //     contractPub
  //   );
  //   try{
  //     const txHash = await pg.program.methods
  //       .redeemClaim()
  //       .accounts({
  //         allConfigNebAccount: allConfigNebAccount,
  //         allConfig: allConfig,
  //         neb: neb,
          
  //         vault:{
  //           userBlackInfo: userBlackInfo,
  //           rewardArray: rewardArray,
  //           allVaultInfo: allVaultInfo,
  //           userVaultInfo: userVaultInfo,
  //         },
  //         userRedeemInfo: userRedeemInfo,
  //         signerAccount: signerAccount,
  //         signer: pg.wallet.publicKey,
  //         systemProgram: web3.SystemProgram.programId,
  //         tokenProgram: tokenProgram,
  //         associatedTokenProgram: associatedTokenProgram,
  //       })
  //       .rpc();
  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

  //   }catch(err){
  //     console.log(err);
  //   }
  // });

  // it("vault", async () => {
  //   let amount = new BN(1000);

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: pg.wallet.publicKey,
  //   });

  //   const [userVaultInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_v_info"), pg.wallet.publicKey.toBuffer()],
  //     contractPub
  //   );

  //   const txHash = await pg.program.methods
  //     .vault(amount)
  //     .accounts({
  //       allConfigNebAccount: allConfigNebAccount,
  //       allConfig: allConfig,
  //       neb: neb,
  //       vault: {
  //         userBlackInfo: userBlackInfo,
  //         rewardArray: rewardArray,
  //         allVaultInfo: allVaultInfo,
  //         userVaultInfo: userVaultInfo,
  //       },
  //       signerAccount: signerAccount,
  //       signer: pg.wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: tokenProgram,
  //       associatedTokenProgram: associatedTokenProgram,
  //     })
  //     .rpc();
  //   console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  // });
  

  // it("unvault", async () => {

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: pg.wallet.publicKey,
  //   });

  //   const [userVaultInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_v_info"), pg.wallet.publicKey.toBuffer()],
  //     contractPub
  //   );

  //   const txHash = await pg.program.methods
  //     .unvault()
  //     .accounts({
  //       allConfigNebAccount: allConfigNebAccount,
  //       allConfig: allConfig,
  //       neb: neb,
  //       vault: {
  //         userBlackInfo: userBlackInfo,
  //         rewardArray: rewardArray,
  //         allVaultInfo: allVaultInfo,
  //         userVaultInfo: userVaultInfo,
  //       },
  //       signerAccount: signerAccount,
  //       signer: pg.wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: tokenProgram,
  //       associatedTokenProgram: associatedTokenProgram,
  //     })
  //     .rpc();
  //   console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  // });

  // it("vault_claim", async () => {

  //   const signerAccount = await anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: wallet.publicKey,
  //   });

  //   const [userVaultInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_v_info"), wallet.publicKey.toBuffer()],
  //     contractPub
  //   );

  //   const txHash = await program.methods
  //     .vaultClaim()
  //     .accounts({
  //       allConfigNebAccount: allConfigNebAccount,
  //       allConfig: allConfig,
  //       neb: neb,
  //       vault: {
  //         userBlackInfo: userBlackInfo,
  //         rewardArray: rewardArray,
  //         allVaultInfo: allVaultInfo,
  //         userVaultInfo: userVaultInfo,
  //       },
  //       signerAccount: signerAccount,
  //       signer: pg.wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: tokenProgram,
  //       associatedTokenProgram: associatedTokenProgram,
  //     })
  //     .rpc();
  //   console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  // });

});
