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

  // =================================================================================================
  // 3. LOAN LIFECYCLE 
  // =================================================================================================
  it("Allows a borrower to request a loan", async () => {
    const loanAmount = new BN(100 * 1_000_000); // 100 tokens
    const durationDays = 30;

    await program.methods
      .requestLoan(
        loanAmount,
        durationDays,
        "Startup capital",
        0 // CollateralType: None
      )
      .accounts({
        borrower: borrower.publicKey,
        loan: loanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrowerProfilePda,
        platform: platformPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([borrower])
      .rpc();

    const loanAccount = await program.account.loan.fetch(loanPda);
    expect(loanAccount.amount.toString()).to.equal(loanAmount.toString());
    assert.ok(loanAccount.status.requested);
  });

  it("Allows the authority to approve a loan", async () => {
    await program.methods
      .approveLoan()
      .accounts({
        authority: authority.publicKey,
        loan: loanPda,
        lendingPool: lendingPoolPda,
        platform: platformPda,
      })
      .rpc();

    const loanAccount = await program.account.loan.fetch(loanPda);
    assert.ok(loanAccount.status.approved);
  });

  it("Allows the authority to disburse the loan", async () => {
    await program.methods
      .disburseLoan()
      .accounts({
        authority: authority.publicKey,
        platform: platformPda,
        loan: loanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrowerProfilePda,
        poolTokenAccount: poolTokenAccount,
        borrowerTokenAccount: borrowerTokenAccount,
        borrower: borrower.publicKey,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const loanAccount = await program.account.loan.fetch(loanPda);
    assert.ok(loanAccount.status.disbursed);
  });

  it("Allows the borrower to make a full payment", async () => {
    // Simple approach - just advance slot by a reasonable amount
    const currentSlot = await context.banksClient.getSlot();
    const slotsToAdvance = 30 * 24 * 60 * 60; // 30 days worth of slots (assuming 1 slot per second)

    // Use warpToSlot with try-catch to handle potential errors
    try {
      await context.warpToSlot(currentSlot + slotsToAdvance);
    } catch (error) {
      console.warn("Time warp failed, continuing with current time:", error);
    }
    const paymentAmount = new BN(100 * 1_000_000); // 100 tokens


    await program.methods
      .repayLoan(paymentAmount)
      .accounts({
        borrower: borrower.publicKey,
        loan: loanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrowerProfilePda,
        borrowerTokenAccount: borrowerTokenAccount,
        poolTokenAccount: poolTokenAccount,
        platform: platformPda,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([borrower])
      .rpc();

    const updatedLoan = await program.account.loan.fetch(loanPda);
    assert.ok(updatedLoan.status.repaid);
    const pool = await program.account.lendingPool.fetch(lendingPoolPda);


    const borrowerProfile = await program.account.userProfile.fetch(borrowerProfilePda);
    expect(borrowerProfile.successfulLoans).to.equal(1);
  });

  // =================================================================================================
  // 4. INTEREST AND WITHDRAWALS 
  // =================================================================================================
  it("Allows a lender to claim earned interest", async () => {
    await program.methods
      .claimInterest()
      .accounts({
        lender: lender.publicKey,
        lenderDeposit: lenderDepositPda,
        lendingPool: lendingPoolPda,
        poolTokenAccount: poolTokenAccount,
        lenderTokenAccount: lenderTokenAccount,
        userProfile: lenderProfilePda,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender])
      .rpc();

    const depositAccount = await program.account.lenderDeposit.fetch(lenderDepositPda);
    expect(depositAccount.interestClaimed.gtn(0)).to.be.true;
  });

  it("Allows a lender to withdraw from the pool", async () => {
    const depositAccountBefore = await program.account.lenderDeposit.fetch(lenderDepositPda);
    const sharesToWithdraw = depositAccountBefore.shares;

    await program.methods
      .withdrawFromPool(sharesToWithdraw)
      .accounts({
        lender: lender.publicKey,
        lenderDeposit: lenderDepositPda,
        lendingPool: lendingPoolPda,
        poolTokenAccount: poolTokenAccount,
        lenderTokenAccount: lenderTokenAccount,
        userProfile: lenderProfilePda,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender])
      .rpc();

    const depositAccountAfter = await program.account.lenderDeposit.fetch(lenderDepositPda);
    expect(depositAccountAfter.shares.eqn(0)).to.be.true;
  });

});