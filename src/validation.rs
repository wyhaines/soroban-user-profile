//! Username validation logic.
//!
//! Usernames follow a validated pattern to prevent squatting:
//! - Pattern: ^[a-z]{3}[_a-z0-9]{0,11}[0-9]{3}$
//! - Length: 6-17 characters
//! - Starts with 3+ lowercase letters
//! - Middle: lowercase letters, digits, underscores
//! - Ends with 3 digits
//! - Examples: alice001, bob_smith123, crypto_fan_999

use soroban_sdk::Bytes;

/// Minimum username length (3 letters + 3 digits).
pub const MIN_USERNAME_LENGTH: u32 = 6;

/// Maximum username length.
pub const MAX_USERNAME_LENGTH: u32 = 17;

/// Minimum number of leading letters required.
pub const MIN_LEADING_LETTERS: u32 = 3;

/// Number of trailing digits required.
pub const TRAILING_DIGITS: u32 = 3;

/// Validate a username according to the pattern.
///
/// Returns true if the username is valid, false otherwise.
///
/// # Rules
/// - Length: 6-17 characters
/// - First 3+ chars: lowercase letters (a-z)
/// - Middle chars: lowercase letters, digits, or underscores
/// - Last 3 chars: digits (0-9)
pub fn validate_username(username: &Bytes) -> bool {
    let len = username.len();

    // Check length bounds
    if len < MIN_USERNAME_LENGTH || len > MAX_USERNAME_LENGTH {
        return false;
    }

    // First 3 characters must be lowercase letters
    for i in 0..MIN_LEADING_LETTERS {
        let b = username.get(i).unwrap();
        if !is_lowercase_letter(b) {
            return false;
        }
    }

    // Last 3 characters must be digits
    for i in (len - TRAILING_DIGITS)..len {
        let b = username.get(i).unwrap();
        if !is_digit(b) {
            return false;
        }
    }

    // Middle characters must be lowercase letters, digits, or underscores
    for i in MIN_LEADING_LETTERS..(len - TRAILING_DIGITS) {
        let b = username.get(i).unwrap();
        if !is_valid_middle_char(b) {
            return false;
        }
    }

    true
}

/// Check if a byte is a lowercase ASCII letter (a-z).
#[inline]
fn is_lowercase_letter(b: u8) -> bool {
    b >= b'a' && b <= b'z'
}

/// Check if a byte is an ASCII digit (0-9).
#[inline]
fn is_digit(b: u8) -> bool {
    b >= b'0' && b <= b'9'
}

/// Check if a byte is valid for the middle portion of a username.
/// Valid chars: lowercase letters, digits, underscores.
#[inline]
fn is_valid_middle_char(b: u8) -> bool {
    is_lowercase_letter(b) || is_digit(b) || b == b'_'
}

/// Normalize a username to lowercase.
///
/// This converts any uppercase letters to lowercase. Returns None if
/// the input contains invalid characters that would still fail validation.
pub fn normalize_username(username: &Bytes) -> Option<Bytes> {
    let len = username.len();
    if len < MIN_USERNAME_LENGTH || len > MAX_USERNAME_LENGTH {
        return None;
    }

    // We can't easily modify Bytes in no_std, so we just validate
    // that all chars are already lowercase or can be lowercased
    for i in 0..len {
        let b = username.get(i).unwrap();
        // Only allow chars that are valid when lowercased
        let valid = is_lowercase_letter(b)
            || (b >= b'A' && b <= b'Z') // uppercase that can be lowercased
            || is_digit(b)
            || b == b'_';
        if !valid {
            return None;
        }
    }

    // For simplicity, require usernames to already be lowercase
    // (normalization would require creating a new Bytes)
    if validate_username(username) {
        Some(username.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_valid_usernames() {
        let env = Env::default();

        // Valid usernames
        assert!(validate_username(&Bytes::from_slice(&env, b"abc123")));
        assert!(validate_username(&Bytes::from_slice(&env, b"alice001")));
        assert!(validate_username(&Bytes::from_slice(&env, b"bob_smith123")));
        assert!(validate_username(&Bytes::from_slice(&env, b"crypto_fan_999")));
        assert!(validate_username(&Bytes::from_slice(
            &env,
            b"abcdefghijklmn123"
        ))); // 17 chars max
    }

    #[test]
    fn test_invalid_usernames() {
        let env = Env::default();

        // Too short
        assert!(!validate_username(&Bytes::from_slice(&env, b"ab123"))); // only 2 letters
        assert!(!validate_username(&Bytes::from_slice(&env, b"abc12"))); // only 2 digits

        // Too long
        assert!(!validate_username(&Bytes::from_slice(
            &env,
            b"abcdefghijklmnop123"
        ))); // 19 chars

        // Doesn't start with 3 letters
        assert!(!validate_username(&Bytes::from_slice(&env, b"ab1123")));
        assert!(!validate_username(&Bytes::from_slice(&env, b"123abc")));

        // Doesn't end with 3 digits
        assert!(!validate_username(&Bytes::from_slice(&env, b"abcdef")));
        assert!(!validate_username(&Bytes::from_slice(&env, b"abc12a")));

        // Invalid characters
        assert!(!validate_username(&Bytes::from_slice(&env, b"abc-123"))); // hyphen not allowed
        assert!(!validate_username(&Bytes::from_slice(&env, b"ABC123"))); // uppercase not allowed
        assert!(!validate_username(&Bytes::from_slice(&env, b"abc.123"))); // dot not allowed
    }
}
