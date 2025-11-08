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

  let depositReceptAccount: PublicKey;
  let withdrawReceptAccount: PublicKey;

  console.log(provider.wallet.publicKey);

  const [escrowPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), provider.wallet.publicKey.toBuffer()],
    MAGIC_PROGRAM_ID  // Use Magic Program ID, not your program ID
  );

  const [escrowAuthPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow_auth"), provider.wallet.publicKey.toBuffer()],
    MAGIC_PROGRAM_ID
  );

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

    [depositReceptAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("deposit_recept"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    [withdrawReceptAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("withdraw_recept"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    console.log(`Liquiidty Pool: ${poolAccount}`);
    console.log(`Liquidity Provider: ${liquidityProviderAccount}`)

    console.log(`Lp Mint: ${lpMint}`);
    console.log(`Provider Lp token account: ${providerLpTokenAccount}`);
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

  it("Add Liquidity OnChain", async () => {

    let depositLiquidityParams = {
      amountA: new anchor.BN(100),
      amountB: new anchor.BN(100),
      minLpTokens: new anchor.BN(100),
      pool: poolAccount,
    }

    const tx = await program.methods.processDepositAddLiquidityOnChain(depositLiquidityParams).accountsPartial({
      provider: provider.wallet.publicKey,
      mintA: mintA,
      mintB: mintB,
      transferAuthority: transferAuthorityAccount,
      lpMint: lpMint,
      tokenVaultA: tokenVaultAaccount,
      tokenVaultB: tokenVaultBaccount,
      providerTokenAAta: providerTokenAccountA,
      providerTokenBAta: providerTokenAccountB,
      providerTokenLpAta: providerLpTokenAccount,
      depositRecept: depositReceptAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Liquidity Provided OnChain: ${tx}`);
  });

  it("Delegate Add Liquidity Recept", async () => {
    let validatorKey = await getClosestValidator(routerConnection);
    let commitFrequency = 30000;

    const tx = await program.methods.processDelegateAddLiquidityReceipt(commitFrequency, validatorKey).accountsPartial({
      provider: provider.wallet.publicKey,
      depositRecept: depositReceptAccount,
    }).transaction();

    const signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);

    console.log(`Transaction Signature: ${signature}`);
  })

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

  // it("Process Mint LP Tokens", async () => {
  //   let lpTokenToMint = new anchor.BN(50);
  //   const tx = await program.methods.processMintLpTokens(lpTokenToMint).accountsPartial({
  //     provider: provider.wallet.publicKey,
  //     transferAuthority: transferAuthorityAccount,
  //     lpMint: lpMint,
  //     providerLpAta: providerLpTokenAccount,
  //     tokenProgram: TOKEN_PROGRAM_ID,
  //     escrow: escrowPda,
  //     escrowAuth: escrowAuthPda,
  //   }).signers([provider.wallet.payer]).rpc();

  //   console.log(`Processed Mint LP Tokens: ${tx}`);
  // })

  it("Commit and Mint LP Tokens", async () => {
    const depositReceptData = await program.account.depositRecept.fetch(depositReceptAccount);
    console.log("LP Tokens to mint:", depositReceptData.lpTokensMinted.toString());

    // let commitAndMintLpTokensParams = {
    //   name: params.name,
    //   provider: provider.wallet.publicKey,
    //   transferAuthority: transferAuthorityAccount,
    //   lpMint: lpMint,
    //   providerLpAta: providerLpTokenAccount,
    // }

    const tx = await program.methods.processCommitAndMintLpTokens().accountsPartial({
      provider: provider.wallet.publicKey,
      pool: poolAccount,
      liquidityProvider: liquidityProviderAccount,
      depositRecept: depositReceptAccount,
      transferAuthority: transferAuthorityAccount,
      lpMint: lpMint,
      providerLpAta: providerLpTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      magicContext: MAGIC_CONTEXT_ID,
      magicProgram: MAGIC_PROGRAM_ID,
    }).transaction();

    const signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);
    console.log(`Committed and Minted LP Tokens!`);
    console.log(`Transaction Signature: ${signature}`);
  });

  it("Process Remove Liquidity OnChain", async () => {
    let removeLiquidityParams = {
      lpTokensToBurn: new anchor.BN(5),
      minAmountA: new anchor.BN(30),
      minAmountB: new anchor.BN(30),
      pool: poolAccount
    };

    let tx = await program.methods.processRemoveLiquidityOnChain(removeLiquidityParams).accountsPartial({
      provider: provider.wallet.publicKey,
      mintA: mintA,
      mintB: mintB,
      transferAuthority: transferAuthorityAccount,
      lpMint: lpMint,
      tokenVaultA: tokenVaultAaccount,
      tokenVaultB: tokenVaultBaccount,
      providerTokenAAta: providerTokenAccountA,
      providerTokenBAta: providerTokenAccountB,
      providerTokenLpAta: providerLpTokenAccount,
      withdrawRecept: withdrawReceptAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Process Delegate Remove Liquidity Receipt", async () => {
    let validatorKey = await getClosestValidator(routerConnection);
    let commitFrequency = 30000;

    const tx = await program.methods.processDelegateRemoveLiquidityReceipt(commitFrequency, validatorKey).accountsPartial({
      provider: provider.wallet.publicKey,
      withdrawRecept: withdrawReceptAccount,
    }).transaction();

    let signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);
    console.log(`Delegated Remove Liquidity Recipt`);
    console.log(`Transaction Signature: ${signature}`);
  });

  it("Process Remove Liquidity ER", async () => {
    let removeLiquidityParams = {
      user: provider.wallet.publicKey,
      lpTokens: new anchor.BN(30),
      minAmountA: new anchor.BN(30),
      minAmountB: new anchor.BN(30),
    };

    const tx = await program.methods.processRemoveLiquidityEr(removeLiquidityParams).accountsPartial({
      provider: provider.wallet.publicKey,
      liquidityProvider: liquidityProviderAccount,
      pool: poolAccount,
      systemProgram: SystemProgram.programId
    }).transaction();

    let signature = await sendMagicTransaction(
      routerConnection,
      tx,
      [provider.wallet.payer]
    );

    await sleepWithAnimation(10);
    console.log(`Removed Liquidity ER`);
    console.log(`Transaction Signature: ${signature}`);
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