/// Program constants for Cerberus Protocol

/// Maximum number of Merkle roots that can be stored in a distributor
pub const MAX_ROOTS: usize = 10;

/// PDA seed for the distributor account
pub const DISTRIBUTOR_SEED: &[u8] = b"distributor";

/// PDA seed for the bitmap account
pub const BITMAP_SEED: &[u8] = b"bitmap";

/// Initial bitmap capacity (in bytes)
/// This can grow dynamically as needed
pub const INITIAL_BITMAP_CAPACITY: usize = 1024;

/// Account discriminator size (Anchor adds this automatically)
pub const DISCRIMINATOR_SIZE: usize = 8;
