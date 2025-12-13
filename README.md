# Cerberus Protocol

**Cerberus Protocol** is a reusable, proof-based distribution and access-control primitive for Solana.

It allows an organizer to commit to a fixed set of eligible wallets off-chain, publish a single Merkle root on-chain, and let users **prove their eligibility and claim exactly once** — without storing the full whitelist on-chain.

Cerberus Protocol is not just an airdrop contract. It can be used for:
- Token airdrops
- Whitelists
- NFT mint access
- DAO voting eligibility
- Any permissioned on-chain action

---

![Cerberus](https://images2.alphacoders.com/138/thumb-1920-1385120.png)

## Core Idea

> **Eligibility is discovered off-chain, but enforced on-chain.**

- The organizer computes a Merkle root from a whitelist.
- The smart contract stores only the root.
- Users obtain a Merkle proof off-chain.
- The contract verifies the proof and allows a one-time claim.

---

## System Overview

Cerberus Protocol has three components:

```
[ Backend (off-chain) ]  --->  [ Smart Contract ]  <---  [ Frontend ]
        (proofs)                 (verification)          (UX)
```

- **Backend** prepares and serves Merkle proofs
- **Smart Contract** verifies proofs and enforces rules
- **Frontend** connects wallets and submits transactions

---

## Organizer Flow (Step by Step)

This is the flow for a team launching an airdrop.

### 1. Prepare Whitelist (Off-chain)

The organizer creates a CSV / JSON file:

```json
[
  { "wallet": "WALLET_1", "amount": 100 },
  { "wallet": "WALLET_2", "amount": 50 }
]
```

This file is never uploaded on-chain.

---

### 2. Build Merkle Tree (Off-chain)

The organizer runs a script that:

- Hashes each entry: `hash(wallet + amount)`
- Builds a Merkle tree
- Outputs:
  - Merkle root
  - Proof + index for each wallet

---

### 3. Deploy & Initialize Contract (On-chain)

The organizer deploys the MerkleGate program and initializes it with:

- Merkle root
- Bitmap size (number of eligible users)

After this step:
- The eligibility set is **locked**
- The organizer cannot change who is eligible

---

### 4. Fund the Airdrop Vault

The organizer mints tokens and transfers them to a vault controlled by the program.

---

## User Flow (Step by Step)

This is the exact experience for an end user.

### 1. User Opens the Website

The frontend prompts the user to connect their wallet.

---

### 2. User Connects Wallet

The frontend now knows the user’s public key.

---

### 3. Frontend Requests Proof (Off-chain)

The frontend calls the backend:

```
GET /proof?wallet=<USER_WALLET>
```

---

### 4. Backend Checks Eligibility

- If the wallet is in the whitelist:
  - Backend returns `{ proof, index, amount }`
- If not:
  - Backend returns `not eligible`

The backend does **not** enforce rules — it only serves data.

---

### 5. Frontend Displays Status

- Eligible → “You can claim X tokens”
- Not eligible → “You are not eligible”

This is how users know their eligibility.

---

### 6. User Submits Claim Transaction

The frontend builds a transaction calling:

```
claim(proof, index, amount)
```

The user signs and submits it.

---

### 7. Smart Contract Verifies (On-chain)

The Cerberus Protocol program:

1. Recomputes the leaf from user data
2. Verifies the Merkle proof against the stored root
3. Checks the bitmap (not already claimed)
4. Marks the index as claimed
5. Executes the action (token transfer / CPI)

---

## Smart Contract Design

### On-chain State

**MerkleDistributor**
- Authority
- One or more Merkle roots
- Bitmap account reference

**ClaimBitmap**
- Bitset tracking which indices have claimed

Each index can be claimed **only once**.

---

### Core Instructions

- `initialize_distributor` – set root(s) and bitmap
- `claim` – verify proof, mark claim, execute action
- `add_root` (optional) – add new eligibility sets
- `update_authority` (optional)

---

## Trust Model

### What the Backend Can Do

- Generate proofs
- Serve proofs
- Go offline

### What the Backend Cannot Do

- Fake eligibility
- Increase claim amounts
- Allow double claims

All enforcement happens on-chain.

---

## Security Properties

- No on-chain whitelist storage
- Proof-based verification
- One-time claim enforced via bitmap
- Backend is untrusted
- Frontend is untrusted

---

## Why Cerberus Protocol

- Gas efficient
- Scales to large airdrops
- Reusable across use cases
- Production-proven pattern

Cerberus Protocol is a **permission engine**, not just an airdrop.

---

## Summary

> Cerberus Protocol lets organizers commit to eligibility off-chain and lets users prove eligibility on-chain — exactly once.

---

## License

MIT

