// import * as nacl from "tweetnacl";
// import * as anchor from "@coral-xyz/anchor";
// import { BN, Program, web3 } from "@coral-xyz/anchor";
// import { HelloAnchor } from "../target/types/hello_anchor";
// import {
//   createCloseAccountInstruction,
//   createBurnInstruction,
//   TOKEN_PROGRAM_ID,
// } from "@solana/spl-token";
// import { sendAndConfirmTransaction } from "@solana/web3.js"; 
// const provider = anchor.AnchorProvider.env();
// anchor.setProvider(provider);
// const program = anchor.workspace.HelloAnchor as Program<HelloAnchor>;
// const wallet = provider.wallet as anchor.Wallet;
// describe("Test", () => {
//     it("Is closed!", async () => {
//         const tokenAccounts = await provider.connection.getParsedTokenAccountsByOwner(
//         wallet.publicKey,
//             {
//                 programId: TOKEN_PROGRAM_ID,
//             }
//         );
//         console.log(tokenAccounts);
    

//         for(let i = 0; i < tokenAccounts.value.length; i++){
//             console.log(tokenAccounts.value[i]);
//         }


//         for (const { pubkey, account } of tokenAccounts.value) {
//             const info = account.data.parsed.info;
//             const tokenAccount = pubkey;
//             const mint = new web3.PublicKey(info.mint);
//             const amount = parseInt(info.tokenAmount.amount);
//             const decimals = info.tokenAmount.decimals;

//             console.log(`🪙 Token Account: ${tokenAccount.toBase58()}`);
//             console.log(`   Mint: ${mint.toBase58()}`);
//             console.log(`   Amount: ${amount / 10 ** decimals}`);

//             const instructions = [];

//             // if amount > 0, need burn first
//             if (amount > 0) {
//                 instructions.push(
//                 createBurnInstruction(
//                     tokenAccount,
//                     mint,
//                     wallet.publicKey,
//                     BigInt(amount),
//                     [],
//                     TOKEN_PROGRAM_ID
//                 )
//                 );
//                 console.log("  🔥 add burn cmd.");
//             }

//             // then close amount, wether have balance or not.
//             instructions.push(
//                 createCloseAccountInstruction(
//                 tokenAccount,
//                 wallet.publicKey, // Send the rent refund to this address
//                 wallet.publicKey,
//                 [],
//                 TOKEN_PROGRAM_ID
//                 )
//             );
//             console.log("  🧼 add close cmd.");

//             // construct transaction
//             const tx = new web3.Transaction().add(...instructions);
//             const sig = await sendAndConfirmTransaction(provider.connection, tx, [wallet.payer]);
//             console.log(`  ✅ finish: ${sig}\n`);
//         }
//     });

// })