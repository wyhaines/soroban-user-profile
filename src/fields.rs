//! Profile field types and standard field definitions.

use soroban_sdk::{contracttype, Address, Bytes, String};

/// Typed field values for extensible profile data.
///
/// This enum allows storing different types of values in profile fields
/// while maintaining type safety.
#[contracttype]
#[derive(Clone, Debug)]
pub enum FieldValue {
    /// String field (bio, avatar URL, etc.)
    StringField(String),

    /// Integer field (age, score, etc.)
    IntField(i128),

    /// Boolean field (verified, available for hiring, etc.)
    BoolField(bool),

    /// Address field (referrer, delegate, etc.)
    AddressField(Address),

    /// Raw bytes field (for custom data)
    BytesField(Bytes),
}

impl FieldValue {
    /// Get the string value if this is a StringField.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            FieldValue::StringField(s) => Some(s),
            _ => None,
        }
    }

    /// Get the integer value if this is an IntField.
    pub fn as_int(&self) -> Option<i128> {
        match self {
            FieldValue::IntField(i) => Some(*i),
            _ => None,
        }
    }

    /// Get the boolean value if this is a BoolField.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::BoolField(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the address value if this is an AddressField.
    pub fn as_address(&self) -> Option<&Address> {
        match self {
            FieldValue::AddressField(a) => Some(a),
            _ => None,
        }
    }

    /// Get the bytes value if this is a BytesField.
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            FieldValue::BytesField(b) => Some(b),
            _ => None,
        }
    }
}

/// Standard field names used by convention.
///
/// These are not enforced by the contract, but provide a consistent
/// naming convention for common profile fields.
pub mod standard_fields {
    /// User biography or description.
    pub const BIO: &str = "bio";

    /// Avatar image URL (IPFS, Gravatar, etc.).
    pub const AVATAR: &str = "avatar";

    /// Personal website or homepage URL.
    pub const HOMEPAGE: &str = "homepage";

    /// Location (city, country, etc.).
    pub const LOCATION: &str = "location";

    /// GitHub username.
    pub const GITHUB: &str = "github";

    /// Twitter/X handle.
    pub const TWITTER: &str = "twitter";

    /// Email address.
    pub const EMAIL: &str = "email";

    /// Whether user is available for hiring.
    pub const AVAILABLE_FOR_HIRING: &str = "hiring";
}
