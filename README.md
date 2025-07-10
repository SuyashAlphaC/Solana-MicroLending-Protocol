# Solana Micro-Lending Protocol 🏦

An on-chain, peer-to-peer micro-lending protocol built on the Solana blockchain. This project provides a framework for financial inclusion by enabling users to create lending pools, build on-chain credit history, and access capital in a transparent and secure environment.

## Table of Contents

- [Solana Micro-Lending Protocol 🏦](#solana-micro-lending-protocol-)
  - [Table of Contents](#table-of-contents)
  - [🧠 Core Concepts](#-core-concepts)
  - [🏗️ Technical Architecture](#️-technical-architecture)
  - [🚀 Getting Started](#-getting-started)
    - [Prerequisites](#prerequisites)
    - [Installation \& Testing](#installation--testing)
      - [Clone the Repository](#clone-the-repository)
      - [Install Dependencies](#install-dependencies)
      - [Build the Program](#build-the-program)
      - [Run the Test Suite](#run-the-test-suite)
  - [📜 Instruction Reference (API)](#-instruction-reference-api)
  - [🔒 Security Considerations](#-security-considerations)
    - [Key security measures implemented:](#key-security-measures-implemented)
  - [🗺️ Roadmap](#️-roadmap)
  - [🤝 Contributing](#-contributing)
  - [📄 License](#-license)

## 🧠 Core Concepts

The protocol is designed around a few key entities that work together to create a decentralized lending ecosystem:

**The Platform**: The central hub governed by a platform authority. It sets the rules, such as transaction fees and loan limits.

**Lending Pools**: Lenders can deposit assets into token-specific pools to provide liquidity. In return for locking their assets, they earn interest from loan repayments.

**Borrowers & User Profiles**: Any user can be a borrower. Each participant has a UserProfile account that serves as their on-chain financial identity, tracking their loan history, repayment rates, and a dynamic credit score.

**On-Chain Credit Score**: A user's reputation is quantified through a credit score calculated from their on-chain activities. This includes successful loan repayments, defaults, social attestations, and verified transaction history.

## 🏗️ Technical Architecture

This protocol is developed using the Anchor framework for secure and rapid development on Solana. The architecture is centered around several key state accounts that manage the protocol's data and logic:

- **Platform**: A singleton account that stores global configuration like fees, loan amount limits, and platform-level statistics.

- **UserProfile**: Stores all data related to a user's reputation, including their credit score, loan history, and verification status (KYC, email, phone).

- **LendingPool**: Manages the liquidity, base interest rates, and loan terms for a specific token asset.

- **Loan**: An account created for each loan, tracking its amount, status (Requested, Approved, Disbursed, Repaid, etc.), due date, and repayment progress.

- **LenderDeposit**: Tracks an individual lender's deposit amount and their corresponding shares within a specific lending pool.

- **SocialAttestation & TransactionHistory**: Accounts that store off-chain data brought on-chain by a trusted authority to help build a user's credit profile.

## 🚀 Getting Started

### Prerequisites

- Rust & Cargo
- Node.js & Yarn (or npm)
- Solana Tool Suite
- Anchor Framework

### Installation & Testing

#### Clone the Repository

```bash
git clone <your-repo-url>
cd <your-repo-name>
```

#### Install Dependencies

```bash
npm install
```

#### Build the Program

This command compiles the Rust code and generates the program's IDL (Interface Definition Language).

```bash
anchor build
```

#### Run the Test Suite

The comprehensive test suite uses anchor-bankrun and solana-bankrun to simulate the Solana runtime and validate all instructions.

```bash
anchor test
```

## 📜 Instruction Reference (API)

The following table details the public instructions available in the protocol and the key accounts required for each.

| Instruction             | Description & Signers                                                                           | Key Accounts (ctx.accounts.*)                                                                                                                    |
| ----------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `initializePlatform`    | (Platform Authority signs) Sets up the platform account and treasury PDA.                       | `platform`, `treasury`, `payer`, `system_program`                                                                                                |
| `initializeUser`        | (User signs) Creates a new user profile with a default credit score.                            | `user_profile`, `user`, `system_program`                                                                                                         |
| `createLendingPool`     | (Pool Authority signs) Creates a new lending pool for a specific token mint.                    | `lending_pool`, `pool_token_account`, `mint`, `authority`, `token_program`, `system_program`                                                     |
| `depositToPool`         | (Lender signs) Allows a lender to deposit assets into a pool to earn interest.                  | `lending_pool`, `mint`, `lender_deposit`, `pool_token_account`, `lender_token_account`, `lender`, `token_program`                                |
| `withdrawFromPool`      | (Lender signs) Allows a lender to withdraw their deposit and earned interest from the pool.     | `lender`, `lending_pool`, `lender_deposit`, `pool_token_account`, `lender_token_account`, `mint`, `token_program`                                |
| `requestLoan`           | (Borrower signs) A user requests a loan from a lending pool, creating a Loan account.           | `platform`, `user_profile`, `lending_pool`, `loan`, `borrower`, `system_program`                                                                 |
| `approveLoan`           | (Pool Authority signs) Approves a loan request and reserves the liquidity in the pool.          | `loan`, `lending_pool`, `authority`                                                                                                              |
| `disburseLoan`          | (Platform Authority signs) Transfers the approved loan amount from the pool to the borrower.    | `authority`, `platform`, `loan`, `mint`, `lending_pool`, `user_profile`, `pool_token_account`, `borrower_token_account`                          |
| `makePayment`           | (Borrower signs) The borrower repays all or part of their loan.                                 | `platform`, `mint`, `loan`, `lending_pool`, `user_profile`, `pool_token_account`, `borrower_token_account`, `treasury_token_account`, `borrower` |
| `claimInterest`         | (Lender signs) A lender claims their proportional share of the interest earned by the pool.     | `lender`, `mint`, `lending_pool`, `lender_deposit`, `pool_token_account`, `lender_token_account`, `token_program`                                |
| `liquidateLoan`         | (Liquidator signs) Marks an overdue loan as defaulted and updates user/platform statistics.     | `liquidator`, `platform`, `loan`, `lending_pool`, `user_profile`                                                                                 |
| `addAttestation`        | (Attester signs) A trusted party adds a social attestation to a user's profile.                 | `attester`, `user`, `user_profile`, `social_attestation`, `system_program`                                                                       |
| `addTransactionHistory` | (Platform Authority signs) Adds a verified transaction history record to a user's profile.      | `authority`, `platform`, `user`, `user_profile`, `transaction_history`, `system_program`                                                         |
| `updateCreditScore`     | (Platform Authority signs) Recalculates a user's credit score based on their on-chain activity. | `authority`, `platform`, `user_profile`, `user`                                                                                                  |

## 🔒 Security Considerations

⚠️ **Important**: This project is for demonstration and educational purposes and has not been audited by a third party. While security best practices have been followed using the Anchor framework, please exercise caution.

### Key security measures implemented:

- **Account Validation**: All instructions use Anchor's constraint system (`has_one`, `seeds`, `bump`) to ensure that only legitimate accounts can be passed in.

- **PDA Ownership**: All state accounts are Program Derived Addresses (PDAs) owned by the program, preventing external modification.

- **Arithmetic Checks**: Using `checked_add` and `checked_sub` to prevent overflows and underflows.

- **Role-Based Access**: Critical functions are restricted to designated signers (e.g., `platform.authority`, `lending_pool.authority`).

## 🗺️ Roadmap

We have a long-term vision for this protocol. Contributions are welcome in the following areas:

- **Advanced Collateralization**: Support for NFTs or other digital assets as loan collateral.

- **Decentralized Governance**: A DAO-based model for managing platform parameters and treasury funds.

- **Off-Chain Data Integration**: Deeper integration with oracle services for more robust credit scoring.

- **Frontend UI**: Building a React-based dApp for interacting with the protocol.

## 🤝 Contributing

We welcome contributions of all kinds! If you're interested in helping, please check out our Contributing Guidelines to get started. All contributors are expected to abide by our Code of Conduct.

## 📄 License

This project is licensed under the Apache License. See the LICENSE file for details.