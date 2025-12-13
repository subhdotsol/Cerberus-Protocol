use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::states::*;
use crate::events::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct Claim<'info> {
    // Step 1: Distributor must exist
    #[account(
        seeds = [b"distributor"],
        bump = distributor.bump
    )]
    pub distributor: Account<'info, MerkleDistributor>,
    
    // Step 2: Bitmap must exist and match distributor
    #[account(
        mut,
        seeds = [b"bitmap", distributor.key().as_ref()],
        bump
    )]
    pub bitmap: Account<'info, ClaimBitmap>,
    
    // Step 3: Vault must match distributor's vault
    /// CHECK: Vault is validated against distributor.vault constraint
    #[account(
        mut,
        constraint = vault.key() == distributor.vault @ CerberusError::VaultMismatch
    )]
    pub vault: UncheckedAccount<'info>,
    
    // Step 4: User's token account to receive tokens
    /// CHECK: User token account is validated by token program during transfer
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    
    // Step 5: Claimer must sign
    pub claimer: Signer<'info>,
    
    // Step 6: Token program for CPI
    pub token_program: Program<'info, Token>,
}

pub fn claim(
    ctx: Context<Claim>,
    root_index: u8,
    leaf_index: u64,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    // Step 1: Get references to accounts
    let distributor = &ctx.accounts.distributor;
    let bitmap = &mut ctx.accounts.bitmap;
    
    // Step 2: Verify root index is valid (within bounds)
    require!(
        (root_index as usize) < distributor.roots.len(),
        CerberusError::InvalidRootIndex
    );
    
    // Step 3: Get the merkle root for this distribution
    let merkle_root = distributor.roots[root_index as usize];
    
    // Step 4: Check if this leaf has already been claimed
    require!(
        !bitmap.is_claimed(leaf_index),
        CerberusError::AlreadyClaimed
    );
    
    // Step 5: Compute the leaf hash from claimer wallet and amount
    // Leaf = keccak256(wallet_pubkey || amount)
    let leaf_hash = solana_program::keccak::hashv(&[
        &ctx.accounts.claimer.key().to_bytes(),
        &amount.to_le_bytes(),
    ]);
    
    // Step 6: Verify the Merkle proof
    let is_valid = verify_merkle_proof(
        &proof,
        merkle_root,
        leaf_hash.0,
    );
    
    // Step 7: If proof is invalid, reject the claim
    require!(is_valid, CerberusError::InvalidProof);
    
    // Step 8: Mark this leaf as claimed in the bitmap
    bitmap.set_claimed(leaf_index);
    
    // Step 9: Prepare token transfer from vault to user
    let seeds = &[
        b"distributor".as_ref(),
        &[distributor.bump],
    ];
    let signer = &[&seeds[..]];
    
    // Step 10: Create CPI context for token transfer
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: distributor.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    // Step 11: Execute the token transfer
    token::transfer(cpi_ctx, amount)?;
    
    // Step 12: Get current timestamp
    let clock = Clock::get()?;
    
    // Step 13: Emit claim event
    emit!(Claimed {
        distributor: distributor.key(),
        claimer: ctx.accounts.claimer.key(),
        root_index,
        leaf_index,
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    // Step 14: Log success message
    msg!(
        "Claim successful - Wallet: {}, Amount: {}, Leaf: {}",
        ctx.accounts.claimer.key(),
        amount,
        leaf_index
    );
    
    Ok(())
}

/// Helper function to verify Merkle proof
fn verify_merkle_proof(
    proof: &[[u8; 32]],
    root: [u8; 32],
    leaf: [u8; 32],
) -> bool {
    // Step 1: Start with the leaf hash
    let mut computed_hash = leaf;
    
    // Step 2: Iterate through each proof element (sibling hash)
    for proof_element in proof.iter() {
        // Step 3: Determine ordering (smaller hash goes first for deterministic hashing)
        computed_hash = if computed_hash <= *proof_element {
            // Step 3a: Current hash is smaller, so it goes first
            solana_program::keccak::hashv(&[
                &computed_hash,
                proof_element,
            ]).0
        } else {
            // Step 3b: Proof element is smaller, so it goes first
            solana_program::keccak::hashv(&[
                proof_element,
                &computed_hash,
            ]).0
        };
    }
    
    // Step 4: Compare computed root with provided root
    computed_hash == root
}
