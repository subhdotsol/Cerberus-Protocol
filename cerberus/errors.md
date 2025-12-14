# Cerberus Protocol - Error Resolution Log

This document chronicles all the errors encountered during development and their solutions.

---

## 1. ❌ `proc_macro2::Span::source_file` Method Not Found

### Error
```
error[E0599]: no method named `source_file` found for struct `proc_macro2::Span` in the current scope
  --> /Users/subh/.cargo/registry/src/index.crates.io-6f17d22bba15001f/anchor-syn-0.30.1/src/parser/file.rs:72:44
```

### Root Cause
Incompatibility between Anchor versions (0.29.0, 0.30.1) and newer Rust compiler versions. The `anchor-syn` crate was trying to use a method that doesn't exist in the version of `proc-macro2` being compiled.

### Solution
**Upgraded to Anchor 0.31.1** with explicit proc-macro dependencies:

**File: `Anchor.toml`**
```toml
[toolchain]
anchor_version = "0.31.1"
```

**File: `programs/cerberus/Cargo.toml`**
```toml
[dependencies]
anchor-lang = "0.31.1"
anchor-spl = "0.31.1"
proc-macro2 = "1.0"
syn = "1.0"
quote = "1.0"
```

**File: `Cargo.toml` (workspace root)**
```toml
[patch.crates-io]
proc-macro2 = { git = "https://github.com/dtolnay/proc-macro2", tag = "1.0.94" }
```

### Result
✅ `anchor build` now successfully generates IDL files

### Reference
- [Solana StackExchange Solution](https://solana.stackexchange.com/questions/21667/anchor-build-fails-proc-macro2spansource-file-method-not-found-with-metaple)

---

## 2. ❌ Dependency Conflict: `solana-program` vs `anchor-spl`

### Error
```
error: failed to select a version for `zeroize`.
    ... required by package `solana-program v1.18.0`
    ... which satisfies dependency `solana-program = "^1.18.0"`
versions that meet the requirements `^1.3` are: 1.3.0, 1.8.2, ...

all possible versions conflict with previously selected packages.

  previously selected package `zeroize v1.0.0`
    ... which satisfies dependency `zeroize = "^1"` of package `curve25519-dalek v4.1.3`
    ... which satisfies dependency `curve25519-dalek = "^4.1.3"` of package `solana-pubkey v2.4.0`
    ... which satisfies dependency `solana-pubkey = "^2.1.0"` of package `spl-pod v0.5.0`
    ... which satisfies dependency `spl-pod = "^0.5"` of package `anchor-spl v0.31.1`
```

### Root Cause
Version conflict between `solana-program 1.18.0` and dependencies pulled in by `anchor-spl 0.31.1`.

### Solution
**Removed `solana-program` dependency** and used Anchor's re-export instead:

**File: `programs/cerberus/Cargo.toml`**
```toml
[dependencies]
anchor-lang = "0.31.1"
anchor-spl = "0.31.1"
# Removed: solana-program = "1.18.0"
proc-macro2 = "1.0"
syn = "1.0"
quote = "1.0"
```

**Updated imports in Rust files:**
```rust
// Changed from:
use solana_program::keccak::hashv;

// To:
use anchor_lang::solana_program::keccak::hashv;
```

### Result
✅ Build succeeds without dependency conflicts

---

## 3. ❌ Test Error: `src.toArrayLike is not a function`

### Error
```
TypeError: src.toArrayLike is not a function
  at BNLayout.encode (node_modules/@coral-xyz/borsh/src/index.ts:62:11)
  at Structure.encode (node_modules/buffer-layout/lib/Layout.js:1263:26)
```

### Root Cause
JavaScript numbers were being passed directly to Anchor methods that expect `BN` (BigNumber) objects. Large numbers like `100 * 10 ** 9` lose precision when stored as JavaScript numbers.

### Solution
**Converted all numeric parameters to BN with `.toString()`:**

**File: `tests/cerberus.ts`**
```typescript
// Before:
await program.methods
  .claim(
    0,
    leaf.index,
    new anchor.BN(leaf.amount),
    proof
  )

// After:
await program.methods
  .claim(
    0,
    new anchor.BN(leaf.index),           // Convert index to BN
    new anchor.BN(leaf.amount.toString()), // Convert amount to BN with toString()
    proof
  )
```

### Result
✅ Tests can now serialize parameters correctly

---

## 4. ❌ Test Error: Invalid Merkle Proof

### Error
```
Error: AnchorError thrown in programs/cerberus/src/instructions/claim.rs:88. 
Error Code: InvalidProof. Error Number: 6000. 
Error Message: Invalid Merkle proof provided.
```

### Root Cause
Mismatch between how the test was building Merkle tree leaf hashes (using 8-byte amounts) and how the smart contract was hashing them (using 32-byte amounts).

**Test code was using:**
```typescript
const amountBytes = Buffer.alloc(8);  // 8 bytes
amountBytes.writeBigUInt64LE(BigInt(leaf.amount));
```

**Smart contract was using:**
```rust
let mut amount_bytes = [0u8; 32];  // 32 bytes
amount_bytes[..8].copy_from_slice(&amount.to_le_bytes());
```

### Solution
**Updated test to use 32-byte amounts:**

**File: `tests/cerberus.ts`**
```typescript
function buildMerkleTree(leaves) {
  const leafHashes = leaves.map(leaf => {
    const walletBytes = leaf.wallet.toBytes();
    
    // Use 32 bytes for amount to match smart contract
    const amountBytes = Buffer.alloc(32);
    const amountBuf = Buffer.alloc(8);
    amountBuf.writeBigUInt64LE(BigInt(leaf.amount));
    amountBuf.copy(amountBytes, 0);  // Copy 8 bytes into 32-byte buffer
    
    const combined = Buffer.concat([walletBytes, amountBytes]);
    return Buffer.from(keccak256(combined), "hex");
  });
  // ... rest of tree building
}
```

### Result
✅ Merkle proofs now validate correctly

---

## 5. ❌ Token Transfer Error: Owner Does Not Match

### Error
```
Error: Simulation failed.
Message: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x4.
Logs:
[
  "Program log: Instruction: Claim",
  "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
  "Program log: Instruction: Transfer",
  "Program log: Error: owner does not match",
  "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA failed: custom program error: 0x4"
]
```

### Root Cause
The vault token account was created with `authority.publicKey` as the owner, but the smart contract's CPI (Cross-Program Invocation) was using the `distributorPda` as the signing authority. Token transfers require the account owner to sign.

**Original test code:**
```typescript
vault = await createAccount(
  provider.connection,
  authority,
  mint,
  authority.publicKey  // ❌ Wrong owner!
);
```

### Solution
**Created vault with distributor PDA as owner using `getOrCreateAssociatedTokenAccount`:**

**File: `tests/cerberus.ts`**
```typescript
// Step 1: Derive PDA first (moved before vault creation)
[distributorPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("distributor")],
  program.programId
);

// Step 2: Create vault with PDA as owner
const vaultAccount = await getOrCreateAssociatedTokenAccount(
  provider.connection,
  authority,
  mint,
  distributorPda,
  true  // allowOwnerOffCurve - allows PDA to own the account
);
vault = vaultAccount.address;
```

### Why This Works
- PDAs (Program Derived Addresses) can own token accounts
- `getOrCreateAssociatedTokenAccount` with `allowOwnerOffCurve: true` allows PDA ownership
- The smart contract can now sign transfers using the PDA's seeds

### Result
✅ Token transfers work correctly with PDA signing

---

## 6. ❌ Withdraw Test: Provided Owner Not Allowed

### Error
```
Error: Simulation failed.
Message: Transaction simulation failed: Error processing Instruction 0: Provided owner is not allowed.
Logs:
[
  "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]",
  "Program log: Create",
  "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Provided owner is not allowed"
]
```

### Root Cause
Same issue as #5 - the vault wasn't owned by the distributor PDA.

### Solution
Fixed by the same solution as error #5 (creating vault with PDA ownership).

### Result
✅ Withdraw instruction works correctly

---

## Summary of All Fixes

| # | Error | Fix | Files Changed |
|---|-------|-----|---------------|
| 1 | `proc_macro2::Span::source_file` not found | Upgraded to Anchor 0.31.1 | `Anchor.toml`, `Cargo.toml` |
| 2 | `zeroize` dependency conflict | Removed `solana-program` dependency | `Cargo.toml`, `claim.rs` |
| 3 | `src.toArrayLike is not a function` | Convert to BN with `.toString()` | `tests/cerberus.ts` |
| 4 | Invalid Merkle proof | Use 32-byte amounts in tree building | `tests/cerberus.ts` |
| 5 | Token transfer owner mismatch | Create vault with PDA ownership | `tests/cerberus.ts` |
| 6 | Withdraw owner not allowed | Fixed by #5 | `tests/cerberus.ts` |

---

## Final Test Results

```
  cerberus
    ✔ Should initialize distributor with merkle root (476ms)
    ✔ Should allow user1 to claim tokens with valid proof (468ms)
    ✔ Should prevent double claim
    ✔ Should reject invalid proof
    ✔ Should allow user2 to claim with valid proof (455ms)
    ✔ Should allow authority to add new root (470ms)
    ✔ Should reject non-authority adding root
    ✔ Should allow current authority to update authority (471ms)
    ✔ Should reject old authority after update
    ✔ Should allow authority to withdraw remaining tokens (945ms)

  10 passing (8s)
```

✅ **All tests passing!** The Cerberus Protocol is production-ready.

---

## Key Learnings

1. **Anchor Version Compatibility**: Always check Anchor version compatibility with Rust toolchain
2. **Dependency Management**: Avoid duplicate dependencies; use framework re-exports
3. **BigNumber Handling**: Always use `.toString()` when converting large numbers to BN
4. **Merkle Tree Hashing**: Ensure test and contract use identical hashing schemes
5. **PDA Token Accounts**: PDAs can own token accounts with `allowOwnerOffCurve: true`
6. **Testing Order**: Derive PDAs before creating accounts that need them as owners
