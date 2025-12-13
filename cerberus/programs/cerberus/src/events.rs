use anchor_lang::prelude::*;

/// Event emitted when a distributor is initialized
#[event]
pub struct DistributorInitialized {
    pub authority: Pubkey,
    pub distributor: Pubkey,
    pub vault: Pubkey,
    pub merkle_root: [u8; 32],
    pub timestamp: i64,
}

/// Event emitted when a new Merkle root is added
#[event]
pub struct RootAdded {
    pub distributor: Pubkey,
    pub root_index: u8,
    pub merkle_root: [u8; 32],
    pub timestamp: i64,
}

/// Event emitted when a user successfully claims their allocation
#[event]
pub struct Claimed {
    pub distributor: Pubkey,
    pub claimer: Pubkey,
    pub root_index: u8,
    pub leaf_index: u64,
    pub amount: u64,
    pub timestamp: i64,
}

/// Event emitted when authority is updated
#[event]
pub struct AuthorityUpdated {
    pub distributor: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when funds are withdrawn from the vault
#[event]
pub struct Withdrawn {
    pub distributor: Pubkey,
    pub authority: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
