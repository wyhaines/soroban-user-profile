//! Event emission helpers for the user profile contract.

use soroban_sdk::{Address, Bytes, Env, Symbol};

/// Emit an event when a profile is registered.
pub fn emit_profile_registered(env: &Env, address: &Address, username: &Bytes) {
    let topics = (Symbol::new(env, "profile_registered"),);
    env.events().publish(topics, (address.clone(), username.clone()));
}

/// Emit an event when a profile is updated.
pub fn emit_profile_updated(env: &Env, address: &Address, field: &Symbol) {
    let topics = (Symbol::new(env, "profile_updated"),);
    env.events().publish(topics, (address.clone(), field.clone()));
}

/// Emit an event when a profile's display name is changed.
pub fn emit_display_name_changed(env: &Env, address: &Address) {
    let topics = (Symbol::new(env, "display_name_changed"),);
    env.events().publish(topics, address.clone());
}

/// Emit an event when a profile is deleted.
pub fn emit_profile_deleted(env: &Env, address: &Address) {
    let topics = (Symbol::new(env, "profile_deleted"),);
    env.events().publish(topics, address.clone());
}

/// Emit an event when a profile is banned by admin.
pub fn emit_profile_banned(env: &Env, address: &Address) {
    let topics = (Symbol::new(env, "profile_banned"),);
    env.events().publish(topics, address.clone());
}

/// Emit an event when a username is transferred.
pub fn emit_username_transferred(env: &Env, username: &Bytes, from: &Address, to: &Address) {
    let topics = (Symbol::new(env, "username_transferred"),);
    env.events().publish(topics, (username.clone(), from.clone(), to.clone()));
}

/// Emit an event when a username is reserved by admin.
pub fn emit_username_reserved(env: &Env, username: &Bytes) {
    let topics = (Symbol::new(env, "username_reserved"),);
    env.events().publish(topics, username.clone());
}

/// Emit an event when a username reservation is released by admin.
pub fn emit_username_unreserved(env: &Env, username: &Bytes) {
    let topics = (Symbol::new(env, "username_unreserved"),);
    env.events().publish(topics, username.clone());
}
