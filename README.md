# soroban-user-profile

[![CI](https://github.com/wyhaines/soroban-user-profile/actions/workflows/ci.yml/badge.svg)](https://github.com/wyhaines/soroban-user-profile/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Global user profile system for Soroban blockchain. Deploy once, use everywhere.

## Overview

This contract provides network-wide user profiles for the Soroban/Stellar ecosystem. Any Soroban application can query and display user profiles, creating a unified identity layer across dApps.

## Features

- **Validated Usernames**: Unique usernames with format validation (e.g., `alice001`, `crypto_fan_999`)
- **Display Names**: Free-form display names for flexibility
- **Extensible Fields**: Key-value profile data (bio, avatar, location, social links, etc.)
- **Embeddable Components**: Render functions for including profile cards in other contracts
- **Soft Deletion**: Profiles can be deleted while preserving username reservation
- **Admin Controls**: Reserved usernames, optional registration fees, abuse prevention

## Username Format

Usernames follow a validated pattern to prevent squatting:

```
^[a-z]{3}[_a-z0-9]{0,11}[0-9]{3}$
```

- 6-17 characters total
- Starts with 3+ lowercase letters
- Middle: lowercase letters, digits, underscores
- Ends with 3 digits
- Examples: `alice001`, `bob_smith123`, `dev_team_999`

## Usage

### Register a Profile

```rust
// Register with username and display name
client.register(&username, &display_name, &caller);
```

### Query Profiles

```rust
// By username
let profile = client.get_by_username(&username);

// By address
let profile = client.get_by_address(&address);

// Get a specific field
let bio = client.get_field(&address, &symbol_short!("bio"));
```

### Update Profile Fields

```rust
client.set_string_field(&symbol_short!("bio"), &bio_text, &caller);
client.set_string_field(&symbol_short!("avatar"), &avatar_url, &caller);
```

### Standard Fields

| Field | Type | Description |
|-------|------|-------------|
| `bio` | String | User biography |
| `avatar` | String | Avatar URL (IPFS, Gravatar, etc.) |
| `homepage` | String | Personal website |
| `location` | String | City, Country |
| `github` | String | GitHub username |
| `twitter` | String | Twitter/X handle |

## Integration

### Include Profile Cards

Other soroban-render contracts can embed profile cards:

```markdown
{{include contract=PROFILE_CONTRACT_ID func="render_profile_card" args="USER_ADDRESS"}}
```

### Compact Author Attribution

```markdown
{{include contract=PROFILE_CONTRACT_ID func="render_profile_card_compact" args="USER_ADDRESS"}}
```

## Render Routes

| Path | Description |
|------|-------------|
| `/` | Home: search form, recent registrations |
| `/u/{username}` | Profile by username |
| `/a/{address}` | Profile by address |
| `/register` | Registration form |
| `/edit` | Edit profile (requires wallet) |
| `/help` | Usage documentation |

## Building

```bash
# Build for testing
cargo build

# Build WASM for deployment
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## Related Projects

- [soroban-render](https://github.com/wyhaines/soroban-render) - Self-rendering Soroban contracts
- [soroban-render-sdk](https://github.com/wyhaines/soroban-render-sdk) - SDK for building renderable contracts
- [soroban-boards](https://github.com/wyhaines/soroban-boards) - Discussion boards using this profile system

## License

Apache 2.0 - see [LICENSE](LICENSE)
