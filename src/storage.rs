//! Storage key definitions for the user profile contract.

use soroban_sdk::{contracttype, Address, Bytes, Symbol};

/// Storage keys for the user profile contract.
///
/// This enum defines all persistent storage keys used by the contract.
#[contracttype]
#[derive(Clone, Debug)]
pub enum ProfileKey {
    /// Contract administrator address.
    Admin,

    /// Total count of registered profiles.
    ProfileCount,

    /// Maps username (Bytes) to owner Address.
    /// Used to enforce username uniqueness.
    Username(Bytes),

    /// Maps Address to Profile struct.
    /// Primary storage for profile data.
    Profile(Address),

    /// Maps (Address, field_name) to FieldValue.
    /// Used for extensible profile fields.
    Field(Address, Symbol),

    /// Reserved usernames that cannot be registered.
    ReservedUsername(Bytes),

    /// Optional registration fee in stroops.
    RegistrationFee,
}

/// Time-to-live for profile data in ledger entries.
pub const PROFILE_TTL_THRESHOLD: u32 = 518400; // ~30 days
pub const PROFILE_TTL_EXTEND: u32 = 2592000; // ~150 days
