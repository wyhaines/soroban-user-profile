//! # Soroban User Profile
//!
//! Global user profile system for the Soroban blockchain ecosystem.
//!
//! This contract provides network-wide user profiles that any Soroban application
//! can query and display. Features include:
//!
//! - Validated unique usernames
//! - Free-form display names
//! - Extensible key-value profile fields
//! - Embeddable render components
//! - Admin controls for moderation
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Register a profile
//! client.register(&username, &display_name, &caller);
//!
//! // Query by username or address
//! let profile = client.get_by_username(&username);
//! let profile = client.get_by_address(&address);
//!
//! // Update fields
//! client.set_string_field(&symbol_short!("bio"), &bio_text, &caller);
//! ```

#![no_std]

mod events;
mod fields;
mod profile;
mod storage;
mod validation;

#[cfg(feature = "render")]
mod render;

pub use fields::{standard_fields, FieldValue};
pub use profile::Profile;
pub use storage::ProfileKey;
pub use validation::{validate_username, MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH};

use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, Map, String, Symbol,
};

use crate::events::*;
use crate::storage::{PROFILE_TTL_EXTEND, PROFILE_TTL_THRESHOLD};

/// Error codes for the user profile contract.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProfileError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized.
    NotInitialized = 2,
    /// Caller is not authorized for this operation.
    NotAuthorized = 3,
    /// Username format is invalid.
    InvalidUsername = 4,
    /// Username is already taken.
    UsernameTaken = 5,
    /// Username is reserved.
    UsernameReserved = 6,
    /// Profile not found.
    ProfileNotFound = 7,
    /// Profile already exists for this address.
    ProfileExists = 8,
    /// Profile has been deleted.
    ProfileDeleted = 9,
    /// Invalid field name.
    InvalidField = 10,
}

impl From<ProfileError> for soroban_sdk::Error {
    fn from(e: ProfileError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct UserProfileContract;

#[contractimpl]
impl UserProfileContract {
    // ========== Initialization ==========

    /// Initialize the contract with an admin address.
    ///
    /// This must be called once before any other operations.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&ProfileKey::Admin) {
            panic_with_error!(&env, ProfileError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&ProfileKey::Admin, &admin);
        env.storage().instance().set(&ProfileKey::ProfileCount, &0u64);
    }

    /// Get the admin address.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ProfileKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::NotInitialized))
    }

    // ========== Registration ==========

    /// Register a new profile with a username.
    ///
    /// # Arguments
    /// * `username` - Validated username (6-17 chars, starts with 3+ letters, ends with 3 digits)
    /// * `display_name` - Free-form display name
    /// * `caller` - Address of the user registering
    ///
    /// # Returns
    /// `true` if registration succeeded
    ///
    /// # Panics
    /// - If username format is invalid
    /// - If username is already taken
    /// - If username is reserved
    /// - If caller already has a profile
    pub fn register(env: Env, username: Bytes, display_name: String, caller: Address) -> bool {
        caller.require_auth();

        // Check contract is initialized
        if !env.storage().instance().has(&ProfileKey::Admin) {
            panic_with_error!(&env, ProfileError::NotInitialized);
        }

        // Validate username format
        if !validation::validate_username(&username) {
            panic_with_error!(&env, ProfileError::InvalidUsername);
        }

        // Check username is not taken
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::Username(username.clone()))
        {
            panic_with_error!(&env, ProfileError::UsernameTaken);
        }

        // Check username is not reserved
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::ReservedUsername(username.clone()))
        {
            panic_with_error!(&env, ProfileError::UsernameReserved);
        }

        // Check caller doesn't already have a profile
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::Profile(caller.clone()))
        {
            panic_with_error!(&env, ProfileError::ProfileExists);
        }

        // Create profile
        let timestamp = env.ledger().sequence() as u64;
        let profile = Profile::new(username.clone(), display_name, caller.clone(), timestamp);

        // Store username -> address mapping
        env.storage()
            .persistent()
            .set(&ProfileKey::Username(username.clone()), &caller);

        // Store profile
        env.storage()
            .persistent()
            .set(&ProfileKey::Profile(caller.clone()), &profile);

        // Extend TTL
        env.storage().persistent().extend_ttl(
            &ProfileKey::Username(username.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );
        env.storage().persistent().extend_ttl(
            &ProfileKey::Profile(caller.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );

        // Increment profile count
        let count: u64 = env
            .storage()
            .instance()
            .get(&ProfileKey::ProfileCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&ProfileKey::ProfileCount, &(count + 1));

        // Emit event
        emit_profile_registered(&env, &caller, &username);

        true
    }

    /// Check if a username is available for registration.
    pub fn is_username_available(env: Env, username: Bytes) -> bool {
        // Check format
        if !validation::validate_username(&username) {
            return false;
        }

        // Check not taken
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::Username(username.clone()))
        {
            return false;
        }

        // Check not reserved
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::ReservedUsername(username.clone()))
        {
            return false;
        }

        true
    }

    // ========== Profile Queries ==========

    /// Get a profile by username.
    pub fn get_by_username(env: Env, username: Bytes) -> Option<Profile> {
        let address: Option<Address> = env
            .storage()
            .persistent()
            .get(&ProfileKey::Username(username));

        match address {
            Some(addr) => {
                let profile: Option<Profile> =
                    env.storage().persistent().get(&ProfileKey::Profile(addr));
                profile.filter(|p| p.is_active())
            }
            None => None,
        }
    }

    /// Get a profile by address.
    pub fn get_by_address(env: Env, address: Address) -> Option<Profile> {
        let profile: Option<Profile> = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(address));
        profile.filter(|p| p.is_active())
    }

    /// Get a profile field value.
    pub fn get_field(env: Env, address: Address, field: Symbol) -> Option<FieldValue> {
        env.storage()
            .persistent()
            .get(&ProfileKey::Field(address, field))
    }

    /// Get all fields for an address.
    ///
    /// Note: This returns only fields that have been explicitly set.
    /// For efficiency, field names must be provided.
    pub fn get_fields(env: Env, address: Address, field_names: soroban_sdk::Vec<Symbol>) -> Map<Symbol, FieldValue> {
        let mut result = Map::new(&env);

        for field in field_names.iter() {
            if let Some(value) = env
                .storage()
                .persistent()
                .get::<_, FieldValue>(&ProfileKey::Field(address.clone(), field.clone()))
            {
                result.set(field, value);
            }
        }

        result
    }

    /// Get total profile count.
    pub fn profile_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&ProfileKey::ProfileCount)
            .unwrap_or(0)
    }

    // ========== Profile Updates ==========

    /// Update the display name.
    pub fn set_display_name(env: Env, display_name: String, caller: Address) {
        caller.require_auth();

        let mut profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(caller.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::ProfileNotFound));

        if profile.deleted {
            panic_with_error!(&env, ProfileError::ProfileDeleted);
        }

        if profile.owner != caller {
            panic_with_error!(&env, ProfileError::NotAuthorized);
        }

        profile.display_name = display_name;
        profile.updated_at = env.ledger().sequence() as u64;

        env.storage()
            .persistent()
            .set(&ProfileKey::Profile(caller.clone()), &profile);

        env.storage().persistent().extend_ttl(
            &ProfileKey::Profile(caller.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );

        emit_display_name_changed(&env, &caller);
    }

    /// Set a string field.
    pub fn set_string_field(env: Env, field: Symbol, value: String, caller: Address) {
        Self::set_field_internal(&env, &caller, field, FieldValue::StringField(value));
    }

    /// Set an integer field.
    pub fn set_int_field(env: Env, field: Symbol, value: i128, caller: Address) {
        Self::set_field_internal(&env, &caller, field, FieldValue::IntField(value));
    }

    /// Set a boolean field.
    pub fn set_bool_field(env: Env, field: Symbol, value: bool, caller: Address) {
        Self::set_field_internal(&env, &caller, field, FieldValue::BoolField(value));
    }

    /// Remove a field.
    pub fn remove_field(env: Env, field: Symbol, caller: Address) {
        caller.require_auth();

        // Verify profile exists and is active
        let profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(caller.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::ProfileNotFound));

        if profile.deleted {
            panic_with_error!(&env, ProfileError::ProfileDeleted);
        }

        if profile.owner != caller {
            panic_with_error!(&env, ProfileError::NotAuthorized);
        }

        env.storage()
            .persistent()
            .remove(&ProfileKey::Field(caller.clone(), field));
    }

    // ========== Profile Management ==========

    /// Soft delete a profile.
    ///
    /// The username remains reserved (cannot be reused by others).
    pub fn delete_profile(env: Env, caller: Address) {
        caller.require_auth();

        let mut profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(caller.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::ProfileNotFound));

        if profile.deleted {
            panic_with_error!(&env, ProfileError::ProfileDeleted);
        }

        if profile.owner != caller {
            panic_with_error!(&env, ProfileError::NotAuthorized);
        }

        profile.deleted = true;
        profile.updated_at = env.ledger().sequence() as u64;

        env.storage()
            .persistent()
            .set(&ProfileKey::Profile(caller.clone()), &profile);

        emit_profile_deleted(&env, &caller);
    }

    /// Transfer profile to a new owner.
    ///
    /// Both old and new owners must authorize.
    pub fn transfer(env: Env, new_owner: Address, caller: Address) {
        caller.require_auth();
        new_owner.require_auth();

        // Get existing profile
        let mut profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(caller.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::ProfileNotFound));

        if profile.deleted {
            panic_with_error!(&env, ProfileError::ProfileDeleted);
        }

        if profile.owner != caller {
            panic_with_error!(&env, ProfileError::NotAuthorized);
        }

        // Check new owner doesn't already have a profile
        if env
            .storage()
            .persistent()
            .has(&ProfileKey::Profile(new_owner.clone()))
        {
            panic_with_error!(&env, ProfileError::ProfileExists);
        }

        let username = profile.username.clone();

        // Update profile ownership
        profile.owner = new_owner.clone();
        profile.updated_at = env.ledger().sequence() as u64;

        // Update username mapping
        env.storage()
            .persistent()
            .set(&ProfileKey::Username(username.clone()), &new_owner);

        // Remove old profile entry
        env.storage()
            .persistent()
            .remove(&ProfileKey::Profile(caller.clone()));

        // Set new profile entry
        env.storage()
            .persistent()
            .set(&ProfileKey::Profile(new_owner.clone()), &profile);

        // Extend TTL
        env.storage().persistent().extend_ttl(
            &ProfileKey::Username(username.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );
        env.storage().persistent().extend_ttl(
            &ProfileKey::Profile(new_owner.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );

        emit_username_transferred(&env, &username, &caller, &new_owner);
    }

    // ========== Admin Functions ==========

    /// Reserve a username (admin only).
    pub fn reserve_username(env: Env, username: Bytes, caller: Address) {
        Self::require_admin(&env, &caller);

        if !validation::validate_username(&username) {
            panic_with_error!(&env, ProfileError::InvalidUsername);
        }

        env.storage()
            .persistent()
            .set(&ProfileKey::ReservedUsername(username.clone()), &true);

        emit_username_reserved(&env, &username);
    }

    /// Release a reserved username (admin only).
    pub fn unreserve_username(env: Env, username: Bytes, caller: Address) {
        Self::require_admin(&env, &caller);

        env.storage()
            .persistent()
            .remove(&ProfileKey::ReservedUsername(username.clone()));

        emit_username_unreserved(&env, &username);
    }

    /// Set the registration fee (admin only).
    pub fn set_registration_fee(env: Env, fee_stroops: i128, caller: Address) {
        Self::require_admin(&env, &caller);

        env.storage()
            .instance()
            .set(&ProfileKey::RegistrationFee, &fee_stroops);
    }

    /// Get the current registration fee.
    pub fn registration_fee(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&ProfileKey::RegistrationFee)
            .unwrap_or(0)
    }

    /// Ban a profile (admin only).
    ///
    /// This soft-deletes the profile.
    pub fn ban_profile(env: Env, address: Address, caller: Address) {
        Self::require_admin(&env, &caller);

        let mut profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(address.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::ProfileNotFound));

        profile.deleted = true;
        profile.updated_at = env.ledger().sequence() as u64;

        env.storage()
            .persistent()
            .set(&ProfileKey::Profile(address.clone()), &profile);

        emit_profile_banned(&env, &address);
    }

    /// Upgrade the contract WASM (admin only).
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ProfileKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, ProfileError::NotInitialized));

        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    // ========== Internal Helpers ==========

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ProfileKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, ProfileError::NotInitialized));

        if *caller != admin {
            panic_with_error!(env, ProfileError::NotAuthorized);
        }

        caller.require_auth();
    }

    fn set_field_internal(env: &Env, caller: &Address, field: Symbol, value: FieldValue) {
        caller.require_auth();

        // Verify profile exists and is active
        let profile: Profile = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(caller.clone()))
            .unwrap_or_else(|| panic_with_error!(env, ProfileError::ProfileNotFound));

        if profile.deleted {
            panic_with_error!(env, ProfileError::ProfileDeleted);
        }

        if profile.owner != *caller {
            panic_with_error!(env, ProfileError::NotAuthorized);
        }

        // Set field
        env.storage()
            .persistent()
            .set(&ProfileKey::Field(caller.clone(), field.clone()), &value);

        // Extend TTL
        env.storage().persistent().extend_ttl(
            &ProfileKey::Field(caller.clone(), field.clone()),
            PROFILE_TTL_THRESHOLD,
            PROFILE_TTL_EXTEND,
        );

        emit_profile_updated(env, caller, &field);
    }
}

// ========== Render Functions ==========

#[cfg(feature = "render")]
#[contractimpl]
impl UserProfileContract {
    /// Main render entry point.
    pub fn render(env: Env, path: Option<String>, viewer: Option<Address>) -> Bytes {
        render::render(&env, path, viewer)
    }

    /// Render a profile card for embedding in other contracts.
    pub fn render_profile_card(env: Env, address: Address) -> Bytes {
        render::render_profile_card(&env, &address)
    }

    /// Render a compact profile card.
    pub fn render_profile_card_compact(env: Env, address: Address) -> Bytes {
        render::render_profile_card_compact(&env, &address)
    }

    /// Render just the username (or truncated address).
    pub fn render_username(env: Env, address: Address) -> Bytes {
        render::render_username(&env, &address)
    }
}
