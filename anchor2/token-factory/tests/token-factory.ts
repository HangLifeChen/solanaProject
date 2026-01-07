import * as anchor from "@coral-xyz/anchor";
import { BN, Program, web3 } from "@coral-xyz/anchor";
import { HelloAnchor } from "../target/types/token_factory";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import * as nacl from "tweetnacl";


describe("Test", () => {

  let randomStr = "es_neb";
  let randomNum1 = 1106;
  let randomNum2 = 2020;
  
   let params = { 
    name: "esNEB", 
    symbol: "esNEB", 
    uri: "https://amethyst-elegant-opossum-120.mypinata.cloud/ipfs/bafkreiaho7ya5v26p5kba7cbzsabsw4c6hiya76hnkhbd3i2ycbvzw7u6e", 
    totalSuply: new BN(100000),
    giveUpAuth: false,
    randomStr: randomStr, 
    randomNum1: new BN(randomNum1), 
    randomNum2: new BN(randomNum2)
  };
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.HelloAnchor as Program<HelloAnchor>;
  const wallet = provider.wallet as NodeWallet;

  function intToUint8Array(number: number) {
    const buffer = new ArrayBuffer(8);
    const dataView = new DataView(buffer);
    // dataView.setInt32(0, number, true);
    dataView.setBigInt64(0, BigInt(number), true);
    return new Uint8Array(buffer);
  }
  console.log("ready const ")
  const contractPub = new web3.PublicKey("Cmbi81Rt4b6z1Cz2Qrzz9ERimFpCNgyvVSMex8ttAfxd");
  
  const [mintPub] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from(randomStr), intToUint8Array(randomNum1), intToUint8Array(randomNum2)],
     program.programId
  );

  const [allConfig] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("all_config")],
     program.programId
  );

  
  console.log("mintPub", mintPub.toBase58());
  const neb = new web3.PublicKey("13ewvwgqDHdDVkT9pbLXJe2T83zodF8LtPHzEB7wpC3o");
  //3tMkrR8M7eNiyfn2wFGGo5k1WDXAKubX6vth51vQAHeh
  //6S5d56HwNmip51YA7iamfDYegJV6wmXxToq1irzvJSEZ
  const esNeb = new web3.PublicKey(mintPub.toBase58());
  

  const systemSigner = new web3.PublicKey("EhuFctMbCSQjZ1EHfZmAqZbnENouizVi8erFyNKaH4ay");

  const systemGetNebAccount = anchor.utils.token.associatedAddress({
    mint: neb,
    owner: systemSigner,
  });
  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  console.log("ready perfect");

  it("create_token", async () => {
   

    const [metadataPub] = web3.PublicKey.findProgramAddressSync(
				[Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mintPub.toBuffer()],
				TOKEN_METADATA_PROGRAM_ID,
			);
``
    const payerTokenAccountPub = await anchor.utils.token.associatedAddress({
      mint: mintPub,
      owner: wallet.publicKey,
    });
    console.log("payerTokenAccountPub:",payerTokenAccountPub.toBase58());
    console.log("mintPub:",mintPub.toBase58());
    console.log("metadataPub:",metadataPub.toBase58())
    try{
      let txHash = await program.methods
      .createToken(params)
      .accounts({
        mint:mintPub,
        metadata:metadataPub,
        signer: wallet.publicKey,
        signerTokenAccount:payerTokenAccountPub,
        rent: new web3.PublicKey("SysvarRent111111111111111111111111111111111"),
        tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        associatedTokenProgram: new web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),        
        systemProgram: web3.SystemProgram.programId,
        tokenMetadataProgram:TOKEN_METADATA_PROGRAM_ID,
      })
      .rpc();

      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    }catch(err){
       console.log(err);
    }
    
  });

  it("config_system", async () => {

    let params = { 
      pauseMint: false, 
      neb: neb, 
      esNeb: esNeb 
    };

    try{
      let txHash = await program.methods
      .configSystem(params)
      .accounts({
        allConfig: allConfig,
        signer: wallet.publicKey,      
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    }catch(err){
      console.log(err);
    }
  });

  // it("black_list", async () => {
  //   //8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1
  //   const blackAddr = new web3.PublicKey("8vg67vNGTawTezVMBYiEeW3d931q1fp73FSFLW3CEKu1");

  //   const [userBlackInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("black_list"), blackAddr.toBuffer()],
  //     program.programId,
  //   );

  //   let txHash = await program.methods
  //     .configBlackList(false)
  //     .accounts({
  //       blackAddr: blackAddr,
  //       userBlackInfo: userBlackInfo,
  //       signer: wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //     })
  //     .rpc();

  //   console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  // });

  // it("mint_token", async () => {

  //   const backendPublicKey  = wallet.publicKey;
  //   const signerPubkey      = wallet.publicKey;
  //   const amount            = 10000000;
  //   const nonce             = 1;
  //   const timestamp         = Math.floor(Date.now() / 10000);

  //   // attach message：pubkey (32) + amount (8) + nonce (8) + timestamp (8)

  //   const message = Buffer.alloc(56);


  //   function u64ToBytesLE(value: bigint): Buffer {
  //     const bytes = Buffer.alloc(8);
  //     let temp = value;
  //     for (let i = 0; i < 8; i++) {
  //       bytes[i] = Number(temp & 0xffn);
  //       temp >>= 8n;
  //     }
  //     return bytes;
  //   }

  //   signerPubkey.toBuffer().copy(message, 0);
  //   u64ToBytesLE(BigInt(amount)).copy(message, 32);
  //   u64ToBytesLE(BigInt(nonce)).copy(message, 40);
  //   u64ToBytesLE(BigInt(timestamp)).copy(message, 48);
    
  //   const signature = nacl.sign.detached(message, wallet.payer.secretKey);

  //   const ed25519Instruction = web3.Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendPublicKey.toBytes(),
  //     message,
  //     signature,
  //   });
  //   const signatureArray = Array.from(signature).map(b => Number(b)); // 转换为 number[]
  //   let params = { 
  //     amount: new BN(amount), 
  //     nonce: new BN(nonce), 
  //     timestamp: new BN(timestamp),
  //     // amount: new BN(123), 
  //     // nonce: new BN(123),
  //     // timestamp: new BN(123), 
  //     signature: signatureArray, 
  //     randomStr: randomStr, 
  //     randomNum1: new BN(randomNum1), 
  //     randomNum2: new BN(randomNum2) 
  //   }

  //   const [userBlackInfo] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("black_list"), wallet.publicKey.toBuffer()],
  //     program.programId,
  //   );

  //   const signerTokenAccount = await anchor.utils.token.associatedAddress({
  //     mint: esNeb,
  //     owner: wallet.publicKey,
  //   });

  //   const [mintCheck] = web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("mint_check"), wallet.publicKey.toBuffer(), intToUint8Array(amount), intToUint8Array(nonce), intToUint8Array(timestamp)],
  //     program.programId,
  //   );
  //   let bytes:Uint8Array= wallet.publicKey.toBytes();
  //   console.log("userBlackInfo:",userBlackInfo.toBase58())
  //   console.log("signer:",wallet.publicKey.toBase58())
  //   console.log("arrary:",bytes)
  //   try{
  //     const verifyAdminIx = await program.methods
  //       .mintToken(params)
  //       .accounts({
  //         allConfig: allConfig,
  //         userBlackInfo: userBlackInfo,
  //         esNeb: esNeb,
  //         mintCheck: mintCheck,
  //         signer: wallet.publicKey,
  //         signerTokenAccount: signerTokenAccount,
  //         systemProgram: web3.SystemProgram.programId,
  //         tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
  //         associatedTokenProgram: new web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),    
  //         instructionSysvar: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
  //       })
  //       .instruction();

  //     const tx = new web3.Transaction().add(ed25519Instruction).add(verifyAdminIx);

  //     // **设置最近区块哈希和手续费付者**
  //     const latestBlockhash = await provider.connection.getLatestBlockhash();
  //     tx.recentBlockhash = latestBlockhash.blockhash;
  //     tx.feePayer = wallet.publicKey;

  //     const signedTx = await wallet.signTransaction(tx);
  //     const txid = await provider.connection.sendRawTransaction(signedTx.serialize());

  //     await provider.connection.confirmTransaction(txid);

  //     console.log("Transaction ID:", txid);
  //   }catch(err){
  //     console.log(err);
  //   }

    
  // });

  // it("change_neb_2_es_neb", async () => {

  //   const signerNebAccount = anchor.utils.token.associatedAddress({
  //     mint: neb,
  //     owner: wallet.publicKey,
  //   });

  //   const signerEsNebAccount = anchor.utils.token.associatedAddress({
  //     mint: esNeb,
  //     owner: wallet.publicKey,
  //   });

  //   let params = { 
  //     amount: new BN(1000), 
  //     randomStr: randomStr, 
  //     randomNum1: new BN(randomNum1), 
  //     randomNum2: new BN(randomNum2) 
  //   };

  //   try{
  //     let txHash = await program.methods
  //     .changeNebToEsNeb(params)
  //     .accounts({
  //       allConfig: allConfig,
  //       systemOwner: systemSigner,
  //       getNebAccount: systemGetNebAccount,
  //       neb: neb,
  //       esNeb: esNeb,
  //       signerNebAccount: signerNebAccount,
  //       signerEsNebAccount: signerEsNebAccount,
  //       signer: wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //       tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
  //       associatedTokenProgram: new web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),    
  //     })
  //     .rpc();

  //     console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

  //   }catch(err){

  //     console.log(err)
  //   }

    
  // });
});

