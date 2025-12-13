use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::states::*;
use crate::events::*;

#[derive(Accounts)]
pub struct InitializeDistributor<'info> {
    // Step 1: Create distributor PDA
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 4 + (32 * 10) + 32 + 32 + 1, // discriminator + authority + vec + roots + vault + bitmap + bump
        seeds = [b"distributor"],
        bump
    )]
    pub distributor: Account<'info, MerkleDistributor>,
    
    // Step 2: Create bitmap PDA
    #[account(
        init,
        payer = authority,
        space = 8 + 4 + 1024, // discriminator + vec length + initial capacity
        seeds = [b"bitmap", distributor.key().as_ref()],
        bump
    )]
    pub bitmap: Account<'info, ClaimBitmap>,
    
    // Step 3: Vault must be a valid token account (unchecked for flexibility)
    /// CHECK: Vault is validated by the authority and used only for storing pubkey
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    
    // Step 4: Authority pays for account creation and signs
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // Step 5: System program for account creation
    pub system_program: Program<'info, System>,
}

pub fn initialize_distributor(
    ctx: Context<InitializeDistributor>,
    merkle_root: [u8; 32],
) -> Result<()> {
    // Step 1: Get mutable reference to distributor account
    let distributor = &mut ctx.accounts.distributor;
    
    // Step 2: Verify signer is the authority (automatically enforced by Anchor)
    // This is implicit - the transaction must be signed by the authority account
    
    // Step 3: Store the authority pubkey
    distributor.authority = ctx.accounts.authority.key();
    
    // Step 4: Initialize the roots vector with the first merkle root
    distributor.roots = vec![merkle_root];
    
    // Step 5: Link the vault token account
    distributor.vault = ctx.accounts.vault.key();
    
    // Step 6: Link the bitmap account
    distributor.bitmap_account = ctx.accounts.bitmap.key();
    
    // Step 7: Store the bump seed for PDA verification
    distributor.bump = ctx.bumps.distributor;
    
    // Step 8: Initialize the bitmap account
    let bitmap = &mut ctx.accounts.bitmap;
    bitmap.claimed = Vec::new(); // Empty bitmap - no claims yet
    
    // Step 9: Get current timestamp
    let clock = Clock::get()?;
    
    // Step 10: Emit initialization event
    emit!(DistributorInitialized {
        authority: ctx.accounts.authority.key(),
        distributor: distributor.key(),
        vault: ctx.accounts.vault.key(),
        merkle_root,
        timestamp: clock.unix_timestamp,
    });
    
    // Step 11: Log success message
    msg!("Distributor initialized with root: {:?}", merkle_root);
    
    Ok(())
}
