use anchor_lang::prelude::*;

#[error_code]
pub enum CerberusError {
    #[msg("Invalid Merkle proof provided")]
    InvalidProof,
    
    #[msg("This allocation has already been claimed")]
    AlreadyClaimed,
    
    #[msg("Invalid root index - root does not exist")]
    InvalidRootIndex,
    
    #[msg("Unauthorized - signer is not the authority")]
    Unauthorized,
    
    #[msg("Maximum number of roots reached (10)")]
    MaxRootsReached,
    
    #[msg("Root already exists in the distributor")]
    RootAlreadyExists,
    
    #[msg("Invalid leaf index - out of bounds")]
    InvalidLeafIndex,
    
    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,
    
    #[msg("Invalid bitmap size - must be greater than 0")]
    InvalidBitmapSize,
    
    #[msg("Vault mismatch - provided vault does not match distributor vault")]
    VaultMismatch,
    
    #[msg("Insufficient vault balance for withdrawal")]
    InsufficientBalance,
}
