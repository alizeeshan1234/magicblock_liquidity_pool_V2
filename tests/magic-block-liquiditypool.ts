import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MagicBlockLiquiditypool } from "../target/types/magic_block_liquiditypool";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { Account, createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, approve } from "@solana/spl-token";
import { sendMagicTransaction, getClosestValidator } from "magic-router-sdk";
import { web3 } from "@coral-xyz/anchor";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import {MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID} from "@magicblock-labs/ephemeral-rollups-sdk";
import { Connection, clusterApiUrl } from "@solana/web3.js";

const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
const DELEGATION_PROGRAM_ID = new PublicKey("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");



describe("magic-block-liquiditypool", () => {
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.magicBlockLiquiditypool as Program<MagicBlockLiquiditypool>;

  const routerConnection = new web3.Connection(
    process.env.ROUTER_ENDPOINT || "https://devnet-router.magicblock.app",
    {
      wsEndpoint: process.env.ROUTER_WS_ENDPOINT || "wss://devnet-router.magicblock.app",
    }
  );

  const params = {
    poolId: new anchor.BN(1),
    name: "SOL-USDC",
    maxAumUsd: new anchor.BN(1000000),
    metadataTitle: "SOL-USDC LP Token",
    metadataSymbol: "SULP",
    metadataUri: "https://example.com/metadata.json",
    tradeFees: 30, // 0.3%
    protocolFees: 10, // 0.1%
    feeRecipient: provider.wallet.publicKey,
  };

  let transferAuthorityAccount: PublicKey;
  let mintA: PublicKey;
  let mintB: PublicKey;
  let lpMint: PublicKey;
  let lpTokenAccount: PublicKey;
  let poolAccount: PublicKey;
  let tokenVaultAaccount: PublicKey;
  let tokenVaultBaccount: PublicKey;
  let metadataAccount: PublicKey;

  let liquidityProviderAccount: PublicKey;
  let providerTokenAccountA: PublicKey;
  let providerTokenAccountB: PublicKey;
  let providerLpTokenAccount: PublicKey;

  console.log(provider.wallet.publicKey);

  before(async () => {
    [transferAuthorityAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("transfer_authority")],
      program.programId
    );

    mintA = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      6
    );

    mintB = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      6
    );

    [lpMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp_token_mint")],
      program.programId
    );

    [lpTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp_token_account")],
      program.programId
    );

    [poolAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), Buffer.from(params.name)],
      program.programId
    );

    [tokenVaultAaccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_account_a"), mintA.toBuffer()],
      program.programId
    );

    [tokenVaultBaccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_account_b"), mintB.toBuffer()],
      program.programId
    );

    [metadataAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        lpMint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    [liquidityProviderAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_provider_account_info"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    let providerTokenAccountAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mintA,
      provider.wallet.publicKey,
    );

    let providerTokenAccountBaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mintB,
      provider.wallet.publicKey,
    );

    providerTokenAccountA = providerTokenAccountAaddress.address;
    providerTokenAccountB = providerTokenAccountBaddress.address;

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mintA,
      providerTokenAccountA,
      provider.wallet.publicKey,
      1000000
    );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mintB,
      providerTokenAccountB,
      provider.wallet.publicKey,
      1000000
    );
  })

  it("Is initialized!", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Initialize Liquidity Pool", async () => {
    const tx = await program.methods.processInitializePool(params).accountsPartial({
      admin: provider.wallet.publicKey,
      transferAuthority: transferAuthorityAccount,
      mintA: mintA,
      mintB: mintB,
      lpMint: lpMint,
      lpTokenAccount: lpTokenAccount,
      pool: poolAccount,
      tokenVaultA: tokenVaultAaccount,
      tokenVaultB: tokenVaultBaccount,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      metadataAccount: metadataAccount,
      metadataProgram: METADATA_PROGRAM_ID
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);

    let providerLpTokenAccountAddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      lpMint,
      provider.wallet.publicKey,
    );
    providerLpTokenAccount = providerLpTokenAccountAddress.address;
  });

  it("Delegate Pool", async () => {
    let validatorKey = await getClosestValidator(routerConnection);
    let commitFrequency = 30000;

    const tx = await program.methods.processDelegatePool(commitFrequency, validatorKey).accountsPartial({
      payer: provider.wallet.publicKey,
      pool: poolAccount,
    }).transaction();

    const signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);

    console.log(`Delegated Pool Account!`);
    console.log(`Delegated Signature: ${signature}`);
  });

  it("Initialize Liquidity Provider", async () => {
    const tx = await program.methods.processInitializeLiquidityProvider().accountsPartial({
      provider: provider.wallet.publicKey,
      liquidityProviderAccountInfo: liquidityProviderAccount,
      systemProgram: SystemProgram.programId,
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Delegate Liquidity Provider", async () => {
    let validatorKey = await getClosestValidator(routerConnection);
    let commitFrequency = 30000;

    const tx = await program.methods.processDelegateLiquidityProvider(commitFrequency, validatorKey).accountsPartial({
      provider: provider.wallet.publicKey,
      liquidityProvider: liquidityProviderAccount
    }).transaction();

    const signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);

    console.log(`Delegated Liquidity Provider Account!`);
    console.log(`Delegated Signature: ${signature}`);
  });

  it("Add Liquidity ER", async () => {
    const addLiquidityParams = {
      user: provider.wallet.publicKey,
      amountA: new anchor.BN(50),
      amountB: new anchor.BN(50),
      minLpTokens: new anchor.BN(50),
    };

    const tx = await program.methods.processAddLiquidityEr(addLiquidityParams).accountsPartial({
      provider: provider.wallet.publicKey,
      liquidityProvider: liquidityProviderAccount,
      pool: poolAccount,
      systemProgram: SystemProgram.programId
    }).transaction();

    const signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    console.log(`Liquidity Provided on ER: ${signature}`);
  });

  it("Commit and Add Liquidity", async () => {
    let commitAndAddLiquidityParams = {
      user: provider.wallet.publicKey,
      amountA: new anchor.BN(50),
      amountB: new anchor.BN(50),
      minLpTokens: new anchor.BN(50),
    };

    // const [delegationRecord] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("delegation"), poolAccount.toBuffer()],
    //   DELEGATION_PROGRAM_ID
    // );

    // const [escrowAuth] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("escrow"), Buffer.from("authority"), delegationRecord.toBuffer()],
    //   DELEGATION_PROGRAM_ID
    // );

    // await approve(
    //   provider.connection,
    //   provider.wallet.payer,
    //   providerTokenAccountA,
    //   escrowAuth, // Use derived escrow_auth
    //   provider.wallet.publicKey,
    //   commitAndAddLiquidityParams.amountA.toNumber()
    // );

    // await approve(
    //   provider.connection,
    //   provider.wallet.payer,
    //   providerTokenAccountB,
    //   escrowAuth,
    //   provider.wallet.publicKey,
    //   commitAndAddLiquidityParams.amountB.toNumber()
    // );

    const tx = await program.methods.processCommitAndAddLiquidity(commitAndAddLiquidityParams).accountsPartial({
      provider: provider.wallet.publicKey,
      mintA: mintA,
      mintB: mintB,
      transferAuthority: transferAuthorityAccount,
      lpMint: lpMint,
      liquidityProvider: liquidityProviderAccount,
      pool: poolAccount,
      providerLpTokenAccount: providerLpTokenAccount,
      providerTokenAAta: providerTokenAccountA,
      providerTokenBAta: providerTokenAccountB,
      tokenVaultA: tokenVaultAaccount,
      tokenVaultB: tokenVaultBaccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      programId: program.programId
    }).transaction();

    let signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);

    console.log(`Transaction Signature: ${signature}`);
    console.log(`Vault A Account: ${tokenVaultAaccount.toBase58()}`);
    console.log(`Vault B Account: ${tokenVaultBaccount.toBase58()}`);

    const connection = new Connection(process.env.ROUTER_ENDPOINT || "https://devnet-router.magicblock.app", "confirmed");
    const tx2 = await connection.getTransaction(signature, { commitment: "confirmed" });
    console.log(tx2.meta.logMessages); 
  })
  
});

async function sleepWithAnimation(seconds: number): Promise<void> {
  const totalMs = seconds * 1000;
  const interval = 500;
  const iterations = Math.floor(totalMs / interval);

  for (let i = 0; i < iterations; i++) {
    const dots = '.'.repeat((i % 3) + 1);
    process.stdout.write(`\rWaiting${dots}   `);
    await new Promise(resolve => setTimeout(resolve, interval));
  }
  process.stdout.write('\r\x1b[K');
}