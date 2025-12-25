//! Integration tests for the user profile contract.

#![cfg(feature = "testutils")]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Symbol};
use soroban_user_profile::{Profile, ProfileError, UserProfileContract, UserProfileContractClient};

fn setup() -> (Env, UserProfileContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UserProfileContract, ());
    let client = UserProfileContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin);

    (env, client, admin)
}

#[test]
fn test_init() {
    let (env, client, admin) = setup();
    assert_eq!(client.admin(), admin);
    assert_eq!(client.profile_count(), 0);
}

#[test]
fn test_register_profile() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    let result = client.register(&username, &display_name, &user);
    assert!(result);

    // Check profile count
    assert_eq!(client.profile_count(), 1);

    // Get profile by username
    let profile = client.get_by_username(&username).unwrap();
    assert_eq!(profile.username, username);
    assert_eq!(profile.display_name, display_name);
    assert_eq!(profile.owner, user);
    assert!(!profile.deleted);

    // Get profile by address
    let profile2 = client.get_by_address(&user).unwrap();
    assert_eq!(profile2.username, username);
}

#[test]
fn test_username_validation() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    // Valid usernames
    assert!(client.is_username_available(&Bytes::from_slice(&env, b"abc123")));
    assert!(client.is_username_available(&Bytes::from_slice(&env, b"alice001")));
    assert!(client.is_username_available(&Bytes::from_slice(&env, b"bob_smith123")));

    // Invalid usernames (format fails)
    assert!(!client.is_username_available(&Bytes::from_slice(&env, b"ab123"))); // too short
    assert!(!client.is_username_available(&Bytes::from_slice(&env, b"ABC123"))); // uppercase
    assert!(!client.is_username_available(&Bytes::from_slice(&env, b"123abc"))); // starts with digits
}

#[test]
fn test_username_uniqueness() {
    let (env, client, _admin) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    // First registration succeeds
    client.register(&username, &display_name, &user1);

    // Username should no longer be available
    assert!(!client.is_username_available(&username));
}

#[test]
fn test_set_fields() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    client.register(&username, &display_name, &user);

    // Set string field
    let bio = String::from_str(&env, "Hello, I am Alice!");
    client.set_string_field(&Symbol::new(&env, "bio"), &bio, &user);

    // Get field
    let field = client.get_field(&user, &Symbol::new(&env, "bio")).unwrap();
    match field {
        soroban_user_profile::FieldValue::StringField(s) => assert_eq!(s, bio),
        _ => panic!("Expected StringField"),
    }

    // Set int field
    client.set_int_field(&Symbol::new(&env, "age"), &25, &user);
    let age = client.get_field(&user, &Symbol::new(&env, "age")).unwrap();
    match age {
        soroban_user_profile::FieldValue::IntField(i) => assert_eq!(i, 25),
        _ => panic!("Expected IntField"),
    }

    // Set bool field
    client.set_bool_field(&Symbol::new(&env, "hiring"), &true, &user);
    let hiring = client.get_field(&user, &Symbol::new(&env, "hiring")).unwrap();
    match hiring {
        soroban_user_profile::FieldValue::BoolField(b) => assert!(b),
        _ => panic!("Expected BoolField"),
    }
}

#[test]
fn test_update_display_name() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");
    let new_display_name = String::from_str(&env, "Alice Smith");

    client.register(&username, &display_name, &user);

    // Update display name
    client.set_display_name(&new_display_name, &user);

    // Verify update
    let profile = client.get_by_address(&user).unwrap();
    assert_eq!(profile.display_name, new_display_name);
}

#[test]
fn test_delete_profile() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    client.register(&username, &display_name, &user);

    // Delete profile
    client.delete_profile(&user);

    // Profile should not be returned (soft deleted)
    assert!(client.get_by_address(&user).is_none());
    assert!(client.get_by_username(&username).is_none());

    // Username should still be unavailable (reserved)
    assert!(!client.is_username_available(&username));
}

#[test]
fn test_transfer_profile() {
    let (env, client, _admin) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    client.register(&username, &display_name, &user1);

    // Transfer to new owner
    client.transfer(&user2, &user1);

    // Old owner should not have profile
    assert!(client.get_by_address(&user1).is_none());

    // New owner should have profile
    let profile = client.get_by_address(&user2).unwrap();
    assert_eq!(profile.owner, user2);
    assert_eq!(profile.username, username);

    // Username still resolves
    let profile_by_name = client.get_by_username(&username).unwrap();
    assert_eq!(profile_by_name.owner, user2);
}

#[test]
fn test_reserve_username() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);
    let reserved_username = Bytes::from_slice(&env, b"stellar123");
    let display_name = String::from_str(&env, "User");

    // Reserve username
    client.reserve_username(&reserved_username, &admin);

    // Username should not be available
    assert!(!client.is_username_available(&reserved_username));
}

#[test]
fn test_unreserve_username() {
    let (env, client, admin) = setup();
    let reserved_username = Bytes::from_slice(&env, b"stellar123");

    // Reserve then unreserve
    client.reserve_username(&reserved_username, &admin);
    client.unreserve_username(&reserved_username, &admin);

    // Username should be available
    assert!(client.is_username_available(&reserved_username));
}

#[test]
fn test_registration_fee() {
    let (env, client, admin) = setup();

    // Set fee
    client.set_registration_fee(&1000, &admin);

    // Verify fee
    assert_eq!(client.registration_fee(), 1000);
}

#[test]
fn test_ban_profile() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    client.register(&username, &display_name, &user);

    // Ban profile
    client.ban_profile(&user, &admin);

    // Profile should not be returned
    assert!(client.get_by_address(&user).is_none());
}

#[test]
fn test_remove_field() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let username = Bytes::from_slice(&env, b"alice001");
    let display_name = String::from_str(&env, "Alice");

    client.register(&username, &display_name, &user);

    // Set and remove field
    let bio = String::from_str(&env, "Hello!");
    client.set_string_field(&Symbol::new(&env, "bio"), &bio, &user);

    // Verify field exists
    assert!(client.get_field(&user, &Symbol::new(&env, "bio")).is_some());

    // Remove field
    client.remove_field(&Symbol::new(&env, "bio"), &user);

    // Field should be gone
    assert!(client.get_field(&user, &Symbol::new(&env, "bio")).is_none());
}
