import * as anchor from '@coral-xyz/anchor';
import { program } from '@coral-xyz/anchor/dist/cjs/native/system';
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { type Connection, Keypair, PublicKey, type Signer } from '@solana/web3.js';
import { BN } from 'bn.js';

export async function sleep(seconds: number) {
  new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

export const generateSeededKeypair = (seed: string) => {
  return Keypair.fromSeed(anchor.utils.bytes.utf8.encode(anchor.utils.sha256.hash(seed)).slice(0, 32));
};

export const expectRevert = async (promise: Promise<any>) => {
  try {
    await promise;
    throw new Error('Expected a revert');
  } catch {
    return;
  }
};

export const mintingTokens = async ({
  connection,
  creator,
  holder = creator,
  mintAKeypair,
  mintBKeypair,
  mintedAmount = 100,
  decimals = 6,
}: {
  connection: Connection;
  creator: Signer;
  holder?: Signer;
  mintAKeypair: Keypair;
  mintBKeypair: Keypair;
  mintedAmount?: number;
  decimals?: number;
}) => {
  console.log("Minting tokens...");
  // Mint tokens
  await createMint(connection, creator, creator.publicKey, creator.publicKey, decimals, mintAKeypair,null,TOKEN_2022_PROGRAM_ID);
  await createMint(connection, creator, creator.publicKey, creator.publicKey, decimals, mintBKeypair,null,TOKEN_2022_PROGRAM_ID);
  console.log("Created mints.");
  await getOrCreateAssociatedTokenAccount(connection, holder, mintAKeypair.publicKey, holder.publicKey, true,null,null,TOKEN_2022_PROGRAM_ID);
  await getOrCreateAssociatedTokenAccount(connection, holder, mintBKeypair.publicKey, holder.publicKey, true,null,null,TOKEN_2022_PROGRAM_ID);
  console.log("Minted tokens.");
  await mintTo(
    connection,
    creator,
    mintAKeypair.publicKey,
    getAssociatedTokenAddressSync(mintAKeypair.publicKey, holder.publicKey, true,TOKEN_2022_PROGRAM_ID),
    creator.publicKey,
    mintedAmount * 10 ** decimals,
    [], // Additional signers
    null, // Commitment
    TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  );
  await mintTo(
    connection,
    creator,
    mintBKeypair.publicKey,
    getAssociatedTokenAddressSync(mintBKeypair.publicKey, holder.publicKey, true,TOKEN_2022_PROGRAM_ID),
    creator.publicKey,
    mintedAmount * 10 ** decimals,
    [], // Additional signers
    null, // Commitment
    TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
  );
  console.log("Minted to holder.");
};

export interface TestValues {
  id: PublicKey;
  fee: number;
  admin: Keypair;
  mintAKeypair: Keypair;
  mintBKeypair: Keypair;
  defaultSupply: anchor.BN;
  ammKey: PublicKey;
  minimumLiquidity: anchor.BN;
  poolKey: PublicKey;
  poolAuthority: PublicKey;
  mintLiquidity: PublicKey;
  depositAmountA: anchor.BN;
  depositAmountB: anchor.BN;
  liquidityAccount: PublicKey;
  poolAccountA: PublicKey;
  poolAccountB: PublicKey;
  holderAccountA: PublicKey;
  holderAccountB: PublicKey;
}

type TestValuesDefaults = {
  [K in keyof TestValues]+?: TestValues[K];
};
export function createValues(defaults?: TestValuesDefaults): TestValues {
  const id = defaults?.id || Keypair.generate().publicKey;
   const provider = anchor.AnchorProvider.env();
  const admin = provider.wallet.payer;
  const ammKey = PublicKey.findProgramAddressSync([id.toBuffer()], anchor.workspace.SwapExample.programId)[0];

  // Making sure tokens are in the right order
  const mintAKeypair = Keypair.generate();
  let mintBKeypair = Keypair.generate();
  while (new BN(mintBKeypair.publicKey.toBytes()).lt(new BN(mintAKeypair.publicKey.toBytes()))) {
    mintBKeypair = Keypair.generate();
  }

  const poolAuthority = PublicKey.findProgramAddressSync(
    [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer(), Buffer.from('authority')],
    anchor.workspace.SwapExample.programId,
  )[0];
  const mintLiquidity = PublicKey.findProgramAddressSync(
    [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer(), Buffer.from('liquidity')],
    anchor.workspace.SwapExample.programId,
  )[0];
  const poolKey = PublicKey.findProgramAddressSync(
    [ammKey.toBuffer(), mintAKeypair.publicKey.toBuffer(), mintBKeypair.publicKey.toBuffer()],
    anchor.workspace.SwapExample.programId,
  )[0];
  
  return {
    id,
    fee: 500,
    admin,
    ammKey,
    mintAKeypair,
    mintBKeypair,
    mintLiquidity,
    poolKey,
    poolAuthority,
    poolAccountA: getAssociatedTokenAddressSync(mintAKeypair.publicKey, poolAuthority, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
    poolAccountB: getAssociatedTokenAddressSync(mintBKeypair.publicKey, poolAuthority, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
    liquidityAccount: getAssociatedTokenAddressSync(mintLiquidity, admin.publicKey, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
    holderAccountA: getAssociatedTokenAddressSync(mintAKeypair.publicKey, admin.publicKey, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
    holderAccountB: getAssociatedTokenAddressSync(mintBKeypair.publicKey, admin.publicKey, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
    depositAmountA: new BN(4 * 10 ** 6),
    depositAmountB: new BN(1 * 10 ** 6),
    minimumLiquidity: new BN(100),
    defaultSupply: new BN(100 * 10 ** 6),
  };
}