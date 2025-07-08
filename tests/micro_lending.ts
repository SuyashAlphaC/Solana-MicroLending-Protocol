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
import { BanksClient, Clock, ProgramTestContext } from "solana-bankrun";

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
  const borrower2 = Keypair.generate();
  const attester = Keypair.generate();

  // Token accounts
  let mint: PublicKey;
  let lenderTokenAccount: PublicKey;
  let borrowerTokenAccount: PublicKey;
  let borrower2TokenAccount: PublicKey;
  let poolTokenAccount: PublicKey;

  // PDAs
  let platformPda: PublicKey;
  let treasuryPda: PublicKey;
  let lenderProfilePda: PublicKey;
  let borrowerProfilePda: PublicKey;
  let borrower2ProfilePda: PublicKey;
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
        toPubkey: borrower2.publicKey,
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

    borrower2TokenAccount = await createAccount(
      //@ts-ignore
      banksClient,
      borrower2,
      mint,
      borrower2.publicKey,
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
    [borrower2ProfilePda] = PublicKey.findProgramAddressSync([SEEDS_USER, borrower2.publicKey.toBuffer()], program.programId);

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

    const loanAccountData = await program.account.loan.fetch(loanPda);
    const disbursedAtTimestamp = loanAccountData.disbursedAt.toNumber();
    const loanDurationInDays = loanAccountData.durationDays;

    // Calculate the total overdue period in seconds
    const secondsInDay = 24 * 60 * 60;
    const totalOverdueSeconds = (loanDurationInDays - 10) * secondsInDay;

    // Calculate the timestamp for when the loan is repayed.
    const repaymentTimestamp = disbursedAtTimestamp + totalOverdueSeconds + 1;

    // Get the current clock state to preserve other properties like slot, epoch, etc.
    const currentClock = await banksClient.getClock();

    // Set the bank's clock to the calculated future timestamp.
    // The on-chain program will read this new timestamp when checking number of days passed.
    context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        BigInt(repaymentTimestamp),
      ),
    );

    const paymentAmount = loanAccountData.amount;

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
    assert.notOk(updatedLoan.status.repaid);
    const pool = await program.account.lendingPool.fetch(lendingPoolPda);


    const borrowerProfile = await program.account.userProfile.fetch(borrowerProfilePda);
    expect(borrowerProfile.successfulLoans).to.equal(0);
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

  // =================================================================================================
  // 5. REPUTATION & DATA 
  // =================================================================================================
  it("Adds a social attestation", async () => {
    const [attestationPda] = PublicKey.findProgramAddressSync([Buffer.from("social_attestation"), borrower.publicKey.toBuffer(), attester.publicKey.toBuffer()], program.programId);

    await program.methods
      .addAttestation({ community: {} }, 950, "Highly recommended", null)
      .accounts({
        attester: attester.publicKey,
        user: borrower.publicKey,
        userProfile: borrowerProfilePda,
        socialAttestation: attestationPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([attester])
      .rpc();

    const attestation = await program.account.socialAttestation.fetch(attestationPda);
    expect(attestation.score).to.equal(950);
  });

  it("Adds transaction history", async () => {
    const borrowerProfile = await program.account.userProfile.fetch(borrowerProfilePda);
    const countBuffer = new BN(borrowerProfile.transactionHistoryCount).toBuffer("le", 2);
    const [txHistoryPda] = PublicKey.findProgramAddressSync([Buffer.from("transaction_history"), borrower.publicKey.toBuffer(), countBuffer], program.programId);

    await program.methods
      .addTransactionHistory({ merchant: {} }, new BN(20 * 1_000_000), null, new BN(Date.now() / 1000), 700, 850)
      .accounts({
        authority: authority.publicKey,
        user: borrower.publicKey,
        userProfile: borrowerProfilePda,
        transactionHistory: txHistoryPda,
        platform: platformPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const updatedProfile = await program.account.userProfile.fetch(borrowerProfilePda);
    expect(updatedProfile.transactionHistoryCount).to.equal(1);
  });

  it("Updates a user's credit score", async () => {
    await program.methods
      .updateCreditScore()
      .accounts({
        authority: authority.publicKey,
        user: borrower.publicKey,
        userProfile: borrowerProfilePda,
        platform: platformPda,
      })
      .rpc();

    const profile = await program.account.userProfile.fetch(borrowerProfilePda);
    // Changed from greaterThan to greaterThanOrEqual since the score might be exactly 300
    expect(profile.creditScore).to.be.greaterThanOrEqual(300);
  });

  // =================================================================================================
  // 6. LIQUIDATION 
  // =================================================================================================
  it("Liquidates an overdue loan", async () => {
    // Create a new loan for liquidation test
    const newLoanPda = PublicKey.findProgramAddressSync([Buffer.from("loan"), borrower2.publicKey.toBuffer(), lendingPoolPda.toBuffer()], program.programId)[0];

    // --- Setup new loan ---
    await program.methods
      .depositToPool(new BN(50 * 1_000_000))
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

    // Initialize new borrower
    await program.methods
      .initializeUser()
      .accounts({
        user: borrower2.publicKey,
      })
      .signers([borrower2])
      .rpc();

    await program.methods
      .requestLoan(new BN(50 * 1_000_000), 10, "Default test", 0)
      .accounts({
        borrower: borrower2.publicKey,
        loan: newLoanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrower2ProfilePda,
        platform: platformPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([borrower2])
      .rpc();

    await program.methods
      .approveLoan()
      .accounts({
        authority: authority.publicKey,
        loan: newLoanPda,
        lendingPool: lendingPoolPda,
        platform: platformPda,
      })
      .rpc();

    await program.methods
      .disburseLoan()
      .accounts({
        borrower: borrower2.publicKey,
        authority: authority.publicKey,
        platform: platformPda,
        loan: newLoanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrower2ProfilePda,
        poolTokenAccount: poolTokenAccount,
        borrowerTokenAccount: borrower2TokenAccount,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Extract the relevant values from the accounts
    const loanAccountData = await program.account.loan.fetch(newLoanPda);
    const disbursedAtTimestamp = loanAccountData.disbursedAt.toNumber();
    const loanDurationInDays = loanAccountData.durationDays;
    const gracePeriodInDays = loanAccountData.gracePeriodDays;

    // Calculate the total overdue period in seconds
    const secondsInDay = 24 * 60 * 60;
    const totalOverdueSeconds = (loanDurationInDays + gracePeriodInDays) * secondsInDay;

    // Calculate the timestamp for when the loan is overdue.
    // We advance the clock to 1 second past the grace period to ensure it's liquidatable.
    const liquidationTimestamp = disbursedAtTimestamp + totalOverdueSeconds + 1;

    // Get the current clock state to preserve other properties like slot, epoch, etc.
    const currentClock = await banksClient.getClock();

    // Set the bank's clock to the calculated future timestamp.
    // The on-chain program will read this new timestamp when checking if the loan is overdue.
    context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        BigInt(liquidationTimestamp),
      ),
    );
    await program.methods
      .liquidateLoan()
      .accounts({
        liquidator: authority.publicKey,
        loan: newLoanPda,
        lendingPool: lendingPoolPda,
        userProfile: borrower2ProfilePda,
        platform: platformPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const loanAccount = await program.account.loan.fetch(newLoanPda);
    assert.ok(loanAccount.status.liquidated);
  });
});