import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { MicroLending } from "../target/types/micro_lending";
import IDL from "../target/idl/micro_lending.json";
import { createAccount, createMint, mintTo } from "spl-token-bankrun";

import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAccount,
  transfer,
} from "@solana/spl-token";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { assert, expect } from "chai";
import { BanksClient, ProgramTestContext } from "solana-bankrun";

describe("micro-lending", () => {
  // Configure the client to use the local cluster.
  let provider: BankrunProvider;
  let program: Program<MicroLending>;
  let context: ProgramTestContext;
  let banksClient: BanksClient;
  let authority: Keypair;
  // Keypairs for test participants
  const lender = Keypair.generate();
  const borrower = Keypair.generate();
  const attester = Keypair.generate();

  // Token accounts
  let mint: PublicKey;
  let lenderTokenAccount: PublicKey;
  let borrowerTokenAccount: PublicKey;
  let poolTokenAccount: PublicKey;

  // PDAs
  let platformPda: PublicKey;
  let treasuryPda: PublicKey;
  let lenderProfilePda: PublicKey;
  let borrowerProfilePda: PublicKey;
  let lendingPoolPda: PublicKey;
  let lenderDepositPda: PublicKey;
  let loanPda: PublicKey;

  // Constants
  const SEEDS_PLATFORM = Buffer.from("platform");
  const SEEDS_TREASURY = Buffer.from("treasury");
  const SEEDS_USER = Buffer.from("user_profile");

  before(async () => {
    // Initialize using startAnchor - this will read your Anchor.toml
    context = await startAnchor("", [], []);
    provider = new BankrunProvider(context);
    program = new Program<MicroLending>(IDL as MicroLending, provider);
    banksClient = context.banksClient;

    authority = provider.wallet.payer;

    // Create transaction for SOL transfers
    const transferTransaction = new Transaction();
    transferTransaction.add(
      SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: lender.publicKey,
        lamports: 1 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: borrower.publicKey,
        lamports: 1 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: attester.publicKey,
        lamports: 0.5 * LAMPORTS_PER_SOL,
      })
    );

    // Get recent blockhash and set it for the transaction
    const recentBlockhash = context.lastBlockhash;
    transferTransaction.recentBlockhash = recentBlockhash;
    transferTransaction.feePayer = authority.publicKey;
    transferTransaction.sign(authority);

    // Process the transaction
    await context.banksClient.processTransaction(transferTransaction);

    // Create a new token mint
    mint = await createMint(
      //@ts-ignore
      banksClient,
      authority,
      authority.publicKey,
      null,
      6
    );

    // Create Associated Token Accounts
    lenderTokenAccount = await createAccount(
      //@ts-ignore
      banksClient,
      lender,
      mint,
      lender.publicKey
    );

    borrowerTokenAccount = await createAccount(
      //@ts-ignore
      banksClient,
      borrower,
      mint,
      borrower.publicKey
    );


    // Mint tokens to lender and borrower
    await mintTo(
      //@ts-ignore
      banksClient,
      authority,
      mint,
      lenderTokenAccount,
      authority,
      1_000_000_000);// 1,000 tokens

    await mintTo(
      //@ts-ignore
      banksClient,
      authority,
      mint,
      borrowerTokenAccount,
      authority,
      1_000_000_000);// 1,000 tokens


    // Derive PDAs
    [platformPda] = PublicKey.findProgramAddressSync([SEEDS_PLATFORM], program.programId);
    [treasuryPda] = PublicKey.findProgramAddressSync([SEEDS_TREASURY], program.programId);
    [lenderProfilePda] = PublicKey.findProgramAddressSync([SEEDS_USER, lender.publicKey.toBuffer()], program.programId);
    [borrowerProfilePda] = PublicKey.findProgramAddressSync([SEEDS_USER, borrower.publicKey.toBuffer()], program.programId);
    [lendingPoolPda] = PublicKey.findProgramAddressSync([Buffer.from("lending_pool"), authority.publicKey.toBuffer(), mint.toBuffer()], program.programId);
    [poolTokenAccount] = PublicKey.findProgramAddressSync([Buffer.from("pool_token_account"), lendingPoolPda.toBuffer()], program.programId);
    [lenderDepositPda] = PublicKey.findProgramAddressSync([Buffer.from("lender_deposit"), lender.publicKey.toBuffer(), lendingPoolPda.toBuffer()], program.programId);
    [loanPda] = PublicKey.findProgramAddressSync([Buffer.from("loan"), borrower.publicKey.toBuffer(), lendingPoolPda.toBuffer()], program.programId);

  });

  // =================================================================================================
  // 1. INITIALIZATION
  // =================================================================================================
  it("Initializes the platform", async () => {
    const [treasury, treasuryBump] = PublicKey.findProgramAddressSync([SEEDS_TREASURY], program.programId);

    await program.methods
      .initializePlatform(
        authority.publicKey,
        treasuryBump,
        100, // 1% platform fee
        new BN(1000 * 1_000_000), // Max loan
        new BN(10 * 1_000_000)   // Min loan
      )
      .accounts({
        payer: authority.publicKey,
      })
      .rpc();

    const platformAccount = await program.account.platform.fetch(platformPda);
    expect(platformAccount.authority.toBase58()).to.equal(authority.publicKey.toBase58());
    expect(platformAccount.platformFee).to.equal(100);
    expect(platformAccount.isActive).to.be.true;
  });

  it("Initializes user profiles for lender and borrower", async () => {
    // Initialize lender
    await program.methods
      .initializeUser()
      .accounts({
        user: lender.publicKey,
      })
      .signers([lender])
      .rpc();

    const lenderProfile = await program.account.userProfile.fetch(lenderProfilePda);
    expect(lenderProfile.owner.toBase58()).to.equal(lender.publicKey.toBase58());

    // Initialize borrower
    await program.methods
      .initializeUser()
      .accounts({
        user: borrower.publicKey,
      })
      .signers([borrower])
      .rpc();

    const borrowerProfile = await program.account.userProfile.fetch(borrowerProfilePda);
    expect(borrowerProfile.owner.toBase58()).to.equal(borrower.publicKey.toBase58());
  });

  // =================================================================================================
  // 2. LENDING POOL MANAGEMENT
  // =================================================================================================
  it("Creates a new lending pool", async () => {
    await program.methods
      .createLendingPool(
        "USDC Main Pool",
        500, // 5% base interest rate
        new BN(365) // 365 days max duration
      )
      .accounts({
        authority: authority.publicKey,
        lendingPool: lendingPoolPda,
        poolTokenAccount: poolTokenAccount,
        mint: mint,
        treasury: treasuryPda,
        platform: platformPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const poolAccount = await program.account.lendingPool.fetch(lendingPoolPda);
    expect(poolAccount.name).to.equal("USDC Main Pool");
    expect(poolAccount.baseInterestRate).to.equal(500);
  });

  it("Allows a lender to deposit into the pool", async () => {
    const depositAmount = new BN(500 * 1_000_000); // 500 tokens

    await program.methods
      .depositToPool(depositAmount)
      .accounts({
        lender: lender.publicKey,
        lenderDeposit: lenderDepositPda,
        lendingPool: lendingPoolPda,
        lenderTokenAccount: lenderTokenAccount,
        poolTokenAccount: poolTokenAccount,
        userProfile: lenderProfilePda,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender])
      .rpc();

    const depositAccount = await program.account.lenderDeposit.fetch(lenderDepositPda);
    const poolAccount = await program.account.lendingPool.fetch(lendingPoolPda);

    expect(depositAccount.amountDeposited.toString()).to.equal(depositAmount.toString());
    expect(poolAccount.availableLiquidity.toString()).to.equal(depositAmount.toString());
  });

});