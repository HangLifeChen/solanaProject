import * as anchor from '@coral-xyz/anchor';
import type { Program } from '@coral-xyz/anchor';
import type NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { ASSOCIATED_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/utils/token';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getMint } from '@solana/spl-token';
import { Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import type { MintNft } from '../target/types/mint_nft';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { mplTokenMetadata, fetchMetadata, findMetadataPda, TokenStandard } from '@metaplex-foundation/mpl-token-metadata';
import { createSignerFromKeypair, publicKey, signerIdentity } from '@metaplex-foundation/umi';
import { dasApi } from '@metaplex-foundation/digital-asset-standard-api';



describe('mint-nft', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

  const wallet = provider.wallet as NodeWallet;

  const program = anchor.workspace.MintNft as Program<MintNft>;

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

  const mintAuthority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('authority')], program.programId)[0];

  const collectionKeypair = Keypair.generate();
  const collectionMint = collectionKeypair.publicKey;

  const mintKeypair = Keypair.generate();
  const mint = mintKeypair.publicKey;

  const umi = createUmi(provider.connection.rpcEndpoint).use(mplTokenMetadata());

  const getMetadata = async (mint: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  const getMasterEdition = async (mint: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  it('Create Collection NFT', async () => {
    console.log('\nCollection Mint Key: ', collectionMint.toBase58());

    const metadata = await getMetadata(collectionMint);
    console.log('Collection Metadata Account: ', metadata.toBase58());

    const masterEdition = await getMasterEdition(collectionMint);
    console.log('Master Edition Account: ', masterEdition.toBase58());

    const destination = getAssociatedTokenAddressSync(collectionMint, wallet.publicKey);
    console.log('Destination ATA = ', destination.toBase58());

    const tx = await program.methods
      .createCollection()
      .accountsPartial({
        user: wallet.publicKey,
        mint: collectionMint,
        mintAuthority,
        metadata,
        masterEdition,
        destination,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc({
        commitment: "confirmed",
      });
    console.log('\nCollection NFT minted: TxID - ', tx);
    
    const umi = createUmi('https://api.devnet.solana.com').use(dasApi())
    const mintAddress = publicKey(collectionMint);
    const result = await umi.rpc.getAsset(mintAddress)
    console.log(result)
  });

  it('Mint NFT', async () => {
    console.log('\nMint', mint.toBase58());

    const metadata = await getMetadata(mint);
    console.log('Metadata', metadata.toBase58());

    const masterEdition = await getMasterEdition(mint);
    console.log('Master Edition', masterEdition.toBase58());

    const destination = getAssociatedTokenAddressSync(mint, wallet.publicKey);
    console.log('Destination', destination.toBase58());

    const tx = await program.methods
      .mintNft()
      .accountsPartial({
        owner: wallet.publicKey,
        destination,
        metadata,
        masterEdition,
        mint,
        mintAuthority,
        collectionMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([mintKeypair])
      .rpc({
          commitment: "confirmed",
      });
    // console.log('\nNFT Minted! Your transaction signature', tx);
    // await provider.connection.confirmTransaction(tx, 'confirmed')
    // const mintUmi = publicKey(mint); // 转 umi key
    // const metadataPda = findMetadataPda(umi, { mint: mintUmi });
    // // const metadataPda = await getMetadata(mint);
    // const metadataUmiKey = publicKey(metadataPda);
    // const tx_metadata = await fetchMetadata(umi, metadataUmiKey);

    // console.log("Metadata object:", tx_metadata);
    // console.log("Name:", tx_metadata.name);
    // console.log("Symbol:", tx_metadata.symbol);
    // console.log("URI:", tx_metadata.uri);

const umi = createUmi('https://api.devnet.solana.com').use(dasApi())
const mintAddress = publicKey(mint);
const result = await umi.rpc.getAsset(mintAddress)
console.log(result)

  });

  it('Verify Collection', async () => {
    const mintMetadata = await getMetadata(mint);
    console.log('\nMint Metadata', mintMetadata.toBase58());

    const collectionMetadata = await getMetadata(collectionMint);
    console.log('Collection Metadata', collectionMetadata.toBase58());

    const collectionMasterEdition = await getMasterEdition(collectionMint);
    console.log('Collection Master Edition', collectionMasterEdition.toBase58());

    const tx = await program.methods
      .verifyCollection()
      .accountsPartial({
        authority: wallet.publicKey,
        metadata: mintMetadata,
        mint,
        mintAuthority,
        collectionMint,
        collectionMetadata,
        collectionMasterEdition,
        systemProgram: SystemProgram.programId,
        sysvarInstruction: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .rpc({
          commitment: "confirmed",
      });
    console.log('\nCollection Verified! Your transaction signature', tx);
    const umi = createUmi('https://api.devnet.solana.com').use(dasApi())
    const mintAddress = publicKey(mint);
    const result = await umi.rpc.getAsset(mintAddress)
    console.log(result)
    await provider.connection.confirmTransaction(tx, 'confirmed');
  });

  // it('burn nft', async () => {
  //   console.log('\nBurning NFT...');
  //   // await sleep(15000); // wait for the transaction to be processed
  //   const myKeypair = umi.eddsa.createKeypairFromSecretKey(wallet.payer.secretKey);
  //   const assetId = publicKey(mint)
  //   // const assetId = publicKey("DEMXziX5aneTDuZnRvGZTbJHyJHihMGJcJdyXt8ePKFo")
  //   // const mint=new PublicKey("")
  //   const umiSigner=createSignerFromKeypair(umi, myKeypair);
  //     umi.use(signerIdentity(umiSigner));
  //     console.log("\numi",umi.identity,"|",umi.payer);
  //     console.log('\nassetId', assetId);
  //     const token = await getAssociatedTokenAddressSync(mint, wallet.publicKey)
  //     const tokenAccount = await getAccount(
  //       program.provider.connection,
  //       token,
  //       "confirmed",
  //   );
  
  //     console.log('\ntoken', tokenAccount);
  //     console.log("ATA owner", tokenAccount.owner.toBase58());
  //     console.log("umiSigner", umiSigner.publicKey);
  //     console.log("ATAaddress",publicKey(token));
  //     console.log("collection", publicKey(collectionMint));


  //     await burnV1(umi, {
  //       mint: assetId,
  //       authority: umiSigner,
  //       tokenOwner:umiSigner.publicKey,
  //       tokenStandard: TokenStandard.NonFungible,
  //       token: publicKey(token),
  //       // if your NFT is part of a collection you will also need to pass in the collection metadata address.
  //       collectionMetadata: findMetadataPda( umi, { mint: publicKey(collectionMint) })
  //     }).sendAndConfirm(umi)

    //    const tokenAccount2 = await getAccount(
    //     program.provider.connection,
    //     token,
    //     "confirmed",
    //  );

    //   console.log('\ntoken', tokenAccount2);
    // });
});