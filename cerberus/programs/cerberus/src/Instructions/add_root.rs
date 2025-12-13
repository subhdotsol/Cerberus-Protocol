use anchor_lang::prelude::*;
use crate::states::*;
use crate::events::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct AddRoot<'info> {
    // Step 1: Distributor must exist and match PDA
    #[account(
        mut,
        seeds = [b"distributor"],
        bump = distributor.bump,
        has_one = authority @ CerberusError::Unauthorized // Ensures signer is the authority
    )]
    pub distributor: Account<'info, MerkleDistributor>,
    
    // Step 2: Authority must sign the transaction
    pub authority: Signer<'info>,
}

pub fn add_root(
    ctx: Context<AddRoot>,
    new_root: [u8; 32],
) -> Result<()> {
    // Step 1: Get mutable reference to distributor
    let distributor = &mut ctx.accounts.distributor;
    
    // Step 2: Verify signer is current authority
    // This is enforced by the `has_one = authority` constraint in the account struct
    
    // Step 3: Check if maximum roots limit reached
    require!(
        distributor.roots.len() < MerkleDistributor::MAX_ROOTS,
        CerberusError::MaxRootsReached
    );
    
    // Step 4: Check if root already exists (prevent duplicates)
    require!(
        !distributor.roots.contains(&new_root),
        CerberusError::RootAlreadyExists
    );
    
    // Step 5: Get the index where new root will be added
    let root_index = distributor.roots.len() as u8;
    
    // Step 6: Append new root to the roots vector
    distributor.roots.push(new_root);
    
    // Step 7: Get current timestamp
    let clock = Clock::get()?;
    
    // Step 8: Emit root added event
    emit!(RootAdded {
        distributor: distributor.key(),
        root_index,
        merkle_root: new_root,
        timestamp: clock.unix_timestamp,
    });
    
    // Step 9: Log success message
    msg!("New root added at index {}: {:?}", root_index, new_root);
    
    Ok(())
}
