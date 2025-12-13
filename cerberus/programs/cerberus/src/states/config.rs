use anchor_lang::prelude::*;

/// Main distributor state account
#[account]
pub struct MerkleDistributor {
    /// Admin authority who controls the distributor
    pub authority: Pubkey,          // 32 bytes
    
    /// Merkle roots (supports multi-root for multiple distributions)
    pub roots: Vec<[u8; 32]>,       // 4 + (32 * MAX_ROOTS) bytes
    
    /// Token vault holding the airdrop funds
    pub vault: Pubkey,              // 32 bytes
    
    /// Bitmap account for tracking claims
    pub bitmap_account: Pubkey,     // 32 bytes
    
    /// Bump seed for PDA verification
    pub bump: u8,                   // 1 byte
}

impl MerkleDistributor {
    /// Maximum number of roots that can be stored
    pub const MAX_ROOTS: usize = 10;
    
    /// Calculate account size for rent
    pub const LEN: usize = 8 +      // discriminator
        32 +                         // authority
        4 + (32 * Self::MAX_ROOTS) + // roots vec (4 bytes length + data)
        32 +                         // vault
        32 +                         // bitmap_account
        1;                           // bump
}

/// Bitmap to track which indices have claimed
#[account]
pub struct ClaimBitmap {
    /// Bitmap data - each bit represents one leaf (1 = claimed, 0 = not claimed)
    pub claimed: Vec<u8>,           // Dynamic size - grows as needed
}

impl ClaimBitmap {
    /// Check if a leaf index has been claimed
    pub fn is_claimed(&self, index: u64) -> bool {
        // Step 1: Calculate which byte contains this bit
        let byte_index = (index / 8) as usize;
        
        // Step 2: Calculate which bit within that byte
        let bit_index = (index % 8) as u8;
        
        // Step 3: Check if byte index is out of bounds
        if byte_index >= self.claimed.len() {
            return false; // Not claimed if bitmap hasn't grown to this index yet
        }
        
        // Step 4: Extract the specific bit
        let byte = self.claimed[byte_index];
        let bit = (byte >> bit_index) & 1;
        
        // Step 5: Return true if bit is 1 (claimed), false if 0 (not claimed)
        bit == 1
    }
    
    /// Mark a leaf index as claimed
    pub fn set_claimed(&mut self, index: u64) {
        // Step 1: Calculate which byte contains this bit
        let byte_index = (index / 8) as usize;
        
        // Step 2: Calculate which bit within that byte
        let bit_index = (index % 8) as u8;
        
        // Step 3: Grow the bitmap if necessary
        while byte_index >= self.claimed.len() {
            self.claimed.push(0); // Add new bytes initialized to 0
        }
        
        // Step 4: Set the specific bit to 1
        self.claimed[byte_index] |= 1 << bit_index;
    }
}