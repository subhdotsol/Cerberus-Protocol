use anchor_lang::prelude::*;
use crate::states::*;
use crate::events::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    // Step 1: Distributor must exist and signer must be current authority
    #[account(
        mut,
        seeds = [b"distributor"],
        bump = distributor.bump,
        has_one = authority @ CerberusError::Unauthorized
    )]
    pub distributor: Account<'info, MerkleDistributor>,
    
    // Step 2: Current authority must sign
    pub authority: Signer<'info>,
}

pub fn update_authority(
    ctx: Context<UpdateAuthority>,
    new_authority: Pubkey,
) -> Result<()> {
    // Step 1: Get mutable reference to distributor
    let distributor = &mut ctx.accounts.distributor;
    
    // Step 2: Verify current signer is the authority
    // This is enforced by the `has_one = authority` constraint
    
    // Step 3: Store the old authority for event logging
    let old_authority = distributor.authority;
    
    // Step 4: Update the authority to the new pubkey
    distributor.authority = new_authority;
    
    // Step 5: Get current timestamp
    let clock = Clock::get()?;
    
    // Step 6: Emit authority updated event
    emit!(AuthorityUpdated {
        distributor: distributor.key(),
        old_authority,
        new_authority,
        timestamp: clock.unix_timestamp,
    });
    
    // Step 7: Log the authority change
    msg!(
        "Authority updated: {} -> {}",
        old_authority,
        new_authority
    );
    
    Ok(())
}
