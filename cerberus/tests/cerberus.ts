import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cerberus } from "../target/types/cerberus";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  createMint, 
  createAccount, 
  mintTo,
  getAccount,
  getOrCreateAssociatedTokenAccount
} from "@solana/spl-token";
import { assert } from "chai";
import { keccak256 } from "js-sha3";

describe("cerberus", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Cerberus as Program<Cerberus>;
  
  // Test accounts
  let mint: PublicKey;
  let vault: PublicKey;
  let distributorPda: PublicKey;
  let bitmapPda: PublicKey;
  let authority: Keypair;
  
  // Test users for claiming
  let user1: Keypair;
  let user1TokenAccount: PublicKey;
  let user2: Keypair;
  let user2TokenAccount: PublicKey;
  
  // Merkle tree data
  let merkleRoot: number[];
  let leaves: { wallet: PublicKey; amount: number; index: number }[];
  let proofs: Map<string, number[][]>;

  before(async () => {
    // Step 1: Create authority keypair
    authority = Keypair.generate();
    
    // Step 2: Airdrop SOL to authority for transaction fees
    const airdropSig = await provider.connection.requestAirdrop(
      authority.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
    
    // Step 3: Create test users
    user1 = Keypair.generate();
    user2 = Keypair.generate();
    
    // Airdrop SOL to users
    const user1Airdrop = await provider.connection.requestAirdrop(
      user1.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(user1Airdrop);
    
    const user2Airdrop = await provider.connection.requestAirdrop(
      user2.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(user2Airdrop);
    
    // Step 4: Create SPL token mint
    mint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      9 // 9 decimals
    );
    
    // Step 5: Derive PDAs (needed for vault creation)
    [distributorPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("distributor")],
      program.programId
    );
    
    [bitmapPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bitmap"), distributorPda.toBuffer()],
      program.programId
    );
    
    // Step 6: Create vault token account (owned by distributor PDA)
    const vaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      distributorPda,
      true  // allowOwnerOffCurve - allows PDA to own the account
    );
    vault = vaultAccount.address;
    
    // Step 7: Mint tokens to vault (1,000,000 tokens)
    await mintTo(
      provider.connection,
      authority,
      mint,
      vault,
      authority,
      1_000_000 * 10 ** 9
    );
    
    // Step 8: Create user token accounts
    user1TokenAccount = await createAccount(
      provider.connection,
      user1,
      mint,
      user1.publicKey
    );
    
    user2TokenAccount = await createAccount(
      provider.connection,
      user2,
      mint,
      user2.publicKey
    );
    
    // Step 9: Build Merkle tree
    leaves = [
      { wallet: user1.publicKey, amount: 100 * 10 ** 9, index: 0 },
      { wallet: user2.publicKey, amount: 200 * 10 ** 9, index: 1 },
    ];
    
    const { root, proofMap } = buildMerkleTree(leaves);
    merkleRoot = root;
    proofs = proofMap;
    
    console.log("Setup complete!");
    console.log("Merkle Root:", Buffer.from(merkleRoot).toString("hex"));
    console.log("Distributor PDA:", distributorPda.toBase58());
    console.log("Bitmap PDA:", bitmapPda.toBase58());
  });

  describe("initialize_distributor", () => {
    it("Should initialize distributor with merkle root", async () => {
      // Step 1: Call initialize_distributor
      const tx = await program.methods
        .initializeDistributor(merkleRoot)
        .accounts({
          distributor: distributorPda,
          bitmap: bitmapPda,
          vault: vault,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();
      
      console.log("Initialize transaction:", tx);
      
      // Step 2: Fetch distributor account
      const distributorAccount = await program.account.merkleDistributor.fetch(
        distributorPda
      );
      
      // Step 3: Verify distributor state
      assert.equal(
        distributorAccount.authority.toBase58(),
        authority.publicKey.toBase58(),
        "Authority mismatch"
      );
      
      assert.equal(
        distributorAccount.vault.toBase58(),
        vault.toBase58(),
        "Vault mismatch"
      );
      
      assert.equal(
        distributorAccount.roots.length,
        1,
        "Should have 1 root"
      );
      
      assert.deepEqual(
        distributorAccount.roots[0],
        merkleRoot,
        "Root mismatch"
      );
      
      // Step 4: Fetch bitmap account
      const bitmapAccount = await program.account.claimBitmap.fetch(bitmapPda);
      
      assert.equal(
        bitmapAccount.claimed.length,
        0,
        "Bitmap should be empty initially"
      );
      
      console.log("✓ Distributor initialized successfully");
    });
  });

  describe("claim", () => {
    it("Should allow user1 to claim tokens with valid proof", async () => {
      // Step 1: Get user1's proof
      const proof = proofs.get(user1.publicKey.toBase58());
      assert.exists(proof, "Proof should exist for user1");
      
      const leaf = leaves[0];
      
      // Step 2: Get vault balance before claim
      const vaultBefore = await getAccount(provider.connection, vault);
      const user1Before = await getAccount(provider.connection, user1TokenAccount);
      
      // Step 3: Call claim instruction
      const tx = await program.methods
        .claim(
          0, // root_index
          new anchor.BN(leaf.index),
          new anchor.BN(leaf.amount.toString()),
          proof
        )
        .accounts({
          distributor: distributorPda,
          bitmap: bitmapPda,
          vault: vault,
          userTokenAccount: user1TokenAccount,
          claimer: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();
      
      console.log("Claim transaction:", tx);
      
      // Step 4: Verify token transfer
      const vaultAfter = await getAccount(provider.connection, vault);
      const user1After = await getAccount(provider.connection, user1TokenAccount);
      
      assert.equal(
        Number(vaultAfter.amount),
        Number(vaultBefore.amount) - leaf.amount,
        "Vault balance should decrease"
      );
      
      assert.equal(
        Number(user1After.amount),
        Number(user1Before.amount) + leaf.amount,
        "User balance should increase"
      );
      
      // Step 5: Verify bitmap updated
      const bitmapAccount = await program.account.claimBitmap.fetch(bitmapPda);
      assert.isTrue(
        isClaimed(bitmapAccount.claimed, leaf.index),
        "Leaf should be marked as claimed"
      );
      
      console.log("✓ User1 claimed successfully");
    });

    it("Should prevent double claim", async () => {
      // Step 1: Try to claim again
      const proof = proofs.get(user1.publicKey.toBase58());
      const leaf = leaves[0];
      
      try {
        await program.methods
          .claim(
            0,
            new anchor.BN(leaf.index),
            new anchor.BN(leaf.amount.toString()),
            proof
          )
          .accounts({
            distributor: distributorPda,
            bitmap: bitmapPda,
            vault: vault,
            userTokenAccount: user1TokenAccount,
            claimer: user1.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user1])
          .rpc();
        
        assert.fail("Should have thrown error for double claim");
      } catch (error) {
        assert.include(
          error.toString(),
          "AlreadyClaimed",
          "Should throw AlreadyClaimed error"
        );
        console.log("✓ Double claim prevented");
      }
    });

    it("Should reject invalid proof", async () => {
      // Step 1: Create invalid proof (all zeros)
      const invalidProof = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];
      const leaf = leaves[1]; // user2's leaf
      
      try {
        await program.methods
          .claim(
            0,
            new anchor.BN(leaf.index),
            new anchor.BN(leaf.amount.toString()),
            invalidProof
          )
          .accounts({
            distributor: distributorPda,
            bitmap: bitmapPda,
            vault: vault,
            userTokenAccount: user2TokenAccount,
            claimer: user2.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user2])
          .rpc();
        
        assert.fail("Should have thrown error for invalid proof");
      } catch (error) {
        assert.include(
          error.toString(),
          "InvalidProof",
          "Should throw InvalidProof error"
        );
        console.log("✓ Invalid proof rejected");
      }
    });

    it("Should allow user2 to claim with valid proof", async () => {
      // Step 1: Get user2's proof
      const proof = proofs.get(user2.publicKey.toBase58());
      const leaf = leaves[1];
      
      // Step 2: Get balances before
      const user2Before = await getAccount(provider.connection, user2TokenAccount);
      
      // Step 3: Claim
      const tx = await program.methods
        .claim(
          0,
          new anchor.BN(leaf.index),
          new anchor.BN(leaf.amount.toString()),
          proof
        )
        .accounts({
          distributor: distributorPda,
          bitmap: bitmapPda,
          vault: vault,
          userTokenAccount: user2TokenAccount,
          claimer: user2.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user2])
        .rpc();
      
      // Step 4: Verify
      const user2After = await getAccount(provider.connection, user2TokenAccount);
      
      assert.equal(
        Number(user2After.amount),
        Number(user2Before.amount) + leaf.amount,
        "User2 balance should increase"
      );
      
      console.log("✓ User2 claimed successfully");
    });
  });

  describe("add_root", () => {
    it("Should allow authority to add new root", async () => {
      // Step 1: Create new merkle root
      const newRoot = Array.from(Buffer.alloc(32, 1)); // Dummy root
      
      // Step 2: Call add_root
      await program.methods
        .addRoot(newRoot)
        .accounts({
          distributor: distributorPda,
          authority: authority.publicKey,
        })
        .signers([authority])
        .rpc();
      
      // Step 3: Verify
      const distributorAccount = await program.account.merkleDistributor.fetch(
        distributorPda
      );
      
      assert.equal(
        distributorAccount.roots.length,
        2,
        "Should have 2 roots now"
      );
      
      assert.deepEqual(
        distributorAccount.roots[1],
        newRoot,
        "New root should match"
      );
      
      console.log("✓ New root added successfully");
    });

    it("Should reject non-authority adding root", async () => {
      const newRoot = Array.from(Buffer.alloc(32, 2));
      
      try {
        await program.methods
          .addRoot(newRoot)
          .accounts({
            distributor: distributorPda,
            authority: user1.publicKey, // Wrong authority
          })
          .signers([user1])
          .rpc();
        
        assert.fail("Should have thrown error for unauthorized");
      } catch (error) {
        assert.include(
          error.toString(),
          "Unauthorized",
          "Should throw Unauthorized error"
        );
        console.log("✓ Non-authority rejected");
      }
    });
  });

  describe("update_authority", () => {
    let newAuthority: Keypair;

    before(async () => {
      newAuthority = Keypair.generate();
      
      // Airdrop SOL to new authority
      const airdrop = await provider.connection.requestAirdrop(
        newAuthority.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdrop);
    });

    it("Should allow current authority to update authority", async () => {
      // Step 1: Update authority
      await program.methods
        .updateAuthority(newAuthority.publicKey)
        .accounts({
          distributor: distributorPda,
          authority: authority.publicKey,
        })
        .signers([authority])
        .rpc();
      
      // Step 2: Verify
      const distributorAccount = await program.account.merkleDistributor.fetch(
        distributorPda
      );
      
      assert.equal(
        distributorAccount.authority.toBase58(),
        newAuthority.publicKey.toBase58(),
        "Authority should be updated"
      );
      
      console.log("✓ Authority updated successfully");
    });

    it("Should reject old authority after update", async () => {
      const dummyRoot = Array.from(Buffer.alloc(32, 3));
      
      try {
        await program.methods
          .addRoot(dummyRoot)
          .accounts({
            distributor: distributorPda,
            authority: authority.publicKey, // Old authority
          })
          .signers([authority])
          .rpc();
        
        assert.fail("Should have thrown error for old authority");
      } catch (error) {
        assert.include(
          error.toString(),
          "Unauthorized",
          "Should throw Unauthorized error"
        );
        console.log("✓ Old authority rejected");
      }
    });

    // Restore authority for withdraw test
    after(async () => {
      await program.methods
        .updateAuthority(authority.publicKey)
        .accounts({
          distributor: distributorPda,
          authority: newAuthority.publicKey,
        })
        .signers([newAuthority])
        .rpc();
    });
  });

  describe("withdraw", () => {
    it("Should allow authority to withdraw remaining tokens", async () => {
      // Step 1: Create recipient token account
      const recipient = await createAccount(
        provider.connection,
        authority,
        mint,
        authority.publicKey
      );
      
      // Step 2: Get vault balance
      const vaultBefore = await getAccount(provider.connection, vault);
      const withdrawAmount = 1000 * 10 ** 9; // Withdraw 1000 tokens
      
      // Step 3: Withdraw
      await program.methods
        .withdraw(new anchor.BN(withdrawAmount.toString()))
        .accounts({
          distributor: distributorPda,
          vault: vault,
          recipient: recipient,
          authority: authority.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();
      
      // Step 4: Verify
      const vaultAfter = await getAccount(provider.connection, vault);
      const recipientAfter = await getAccount(provider.connection, recipient);
      
      assert.equal(
        Number(vaultAfter.amount),
        Number(vaultBefore.amount) - withdrawAmount,
        "Vault should decrease"
      );
      
      assert.equal(
        Number(recipientAfter.amount),
        withdrawAmount,
        "Recipient should receive tokens"
      );
      
      console.log("✓ Withdrawal successful");
    });
  });
});

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Build Merkle tree from leaves
 */
function buildMerkleTree(leaves: { wallet: PublicKey; amount: number; index: number }[]) {
  // Step 1: Create leaf hashes
  const leafHashes = leaves.map(leaf => {
    const walletBytes = leaf.wallet.toBytes();
    // Use 32 bytes for amount to match smart contract
    const amountBytes = Buffer.alloc(32);
    const amountBuf = Buffer.alloc(8);
    amountBuf.writeBigUInt64LE(BigInt(leaf.amount));
    amountBuf.copy(amountBytes, 0);
    
    const combined = Buffer.concat([walletBytes, amountBytes]);
    return Buffer.from(keccak256(combined), "hex");
  });
  
  // Step 2: Build tree layers
  let currentLayer = leafHashes;
  const proofMap = new Map<string, number[][]>();
  
  // Store proofs for each leaf
  leaves.forEach((leaf, index) => {
    proofMap.set(leaf.wallet.toBase58(), []);
  });
  
  while (currentLayer.length > 1) {
    const nextLayer: Buffer[] = [];
    
    for (let i = 0; i < currentLayer.length; i += 2) {
      const left = currentLayer[i];
      const right = i + 1 < currentLayer.length ? currentLayer[i + 1] : left;
      
      // Store sibling for proof
      leaves.forEach((leaf, leafIndex) => {
        if (leafIndex === i) {
          proofMap.get(leaf.wallet.toBase58())!.push(Array.from(right));
        } else if (leafIndex === i + 1) {
          proofMap.get(leaf.wallet.toBase58())!.push(Array.from(left));
        }
      });
      
      // Hash pair (sorted)
      const pair = left.compare(right) <= 0 
        ? Buffer.concat([left, right])
        : Buffer.concat([right, left]);
      
      nextLayer.push(Buffer.from(keccak256(pair), "hex"));
    }
    
    currentLayer = nextLayer;
  }
  
  const root = Array.from(currentLayer[0]);
  
  return { root, proofMap };
}

/**
 * Check if a leaf index is claimed in bitmap
 */
function isClaimed(bitmap: number[], index: number): boolean {
  const byteIndex = Math.floor(index / 8);
  const bitIndex = index % 8;
  
  if (byteIndex >= bitmap.length) {
    return false;
  }
  
  return ((bitmap[byteIndex] >> bitIndex) & 1) === 1;
}
