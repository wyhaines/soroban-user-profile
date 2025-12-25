//! Profile struct and related types.

use soroban_sdk::{contracttype, Address, Bytes, String};

/// User profile metadata.
///
/// This struct contains the core profile information that is stored
/// for each registered user.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Profile {
    /// Unique validated username (lowercase, validated format).
    pub username: Bytes,

    /// Free-form display name (any UTF-8 string).
    pub display_name: String,

    /// Profile owner's blockchain address.
    pub owner: Address,

    /// Timestamp when the profile was created (ledger sequence).
    pub created_at: u64,

    /// Timestamp when the profile was last updated (ledger sequence).
    pub updated_at: u64,

    /// Soft deletion flag. Deleted profiles keep their username reserved.
    pub deleted: bool,
}

impl Profile {
    /// Create a new profile.
    pub fn new(
        username: Bytes,
        display_name: String,
        owner: Address,
        created_at: u64,
    ) -> Self {
        Self {
            username,
            display_name,
            owner,
            created_at,
            updated_at: created_at,
            deleted: false,
        }
    }

    /// Check if this profile is active (not deleted).
    pub fn is_active(&self) -> bool {
        !self.deleted
    }
}
