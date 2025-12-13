use anchor_lang::prelude::*;

declare_id!("HopC35nDjfRjRGYEvao9y3j3EN2iqtqKJ6Zkj6MaeshD");

// Import all modules
pub mod errors;
pub mod events;
pub mod states;
pub mod instructions;
pub mod constants;

// Re-export for convenience
use errors::*;
use events::*;
use states::*;
use instructions::*;

#[program]
pub mod cerberus {
    use super::*;

    /// Initialize a new Merkle distributor
    pub fn initialize_distributor(
        ctx: Context<InitializeDistributor>,
        merkle_root: [u8; 32],
    ) -> Result<()> {
        instructions::initialize_distributor(ctx, merkle_root)
    }

    /// Add a new Merkle root for multi-distribution support
    pub fn add_root(
        ctx: Context<AddRoot>,
        new_root: [u8; 32],
    ) -> Result<()> {
        instructions::add_root(ctx, new_root)
    }

    /// Claim tokens with Merkle proof verification
    pub fn claim(
        ctx: Context<Claim>,
        root_index: u8,
        leaf_index: u64,
        amount: u64,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::claim(ctx, root_index, leaf_index, amount, proof)
    }

    /// Update the distributor authority
    pub fn update_authority(
        ctx: Context<UpdateAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::update_authority(ctx, new_authority)
    }

    /// Emergency withdrawal of unclaimed funds
    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
    ) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }
}
