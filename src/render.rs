//! Render functions for the user profile contract.

use soroban_sdk::{Address, Bytes, Env, String, Symbol};
use soroban_render_sdk::prelude::*;

use crate::fields::FieldValue;
use crate::profile::Profile;
use crate::storage::ProfileKey;

/// Main render entry point with routing.
pub fn render(env: &Env, path: Option<String>, viewer: Option<Address>) -> Bytes {
    Router::new(env, path)
        .handle(b"/", |_| render_home(env, &viewer))
        .or_handle(b"/register", |_| render_register_form(env, &viewer))
        .or_handle(b"/edit", |_| render_edit_form(env, &viewer))
        .or_handle(b"/help", |_| render_help(env))
        .or_handle(b"/u/*", |req| {
            let username = req.get_wildcard().unwrap_or_else(|| Bytes::new(env));
            render_profile_by_username(env, &username, &viewer)
        })
        .or_handle(b"/a/*", |req| {
            let _addr_str = req.get_wildcard().unwrap_or_else(|| Bytes::new(env));
            render_profile_by_address_str(env, &viewer)
        })
        .or_default(|_| render_home(env, &viewer))
}

/// Render the home page with search form.
fn render_home(env: &Env, viewer: &Option<Address>) -> Bytes {
    let mut md = MarkdownBuilder::new(env)
        .h1("User Profiles")
        .paragraph("Global identity for Soroban applications.");

    // Profile count
    let count: u64 = env
        .storage()
        .instance()
        .get(&ProfileKey::ProfileCount)
        .unwrap_or(0);

    md = md
        .text("**Registered profiles:** ")
        .raw(u64_to_bytes(env, count))
        .newline()
        .newline();

    // Search form
    md = md
        .h2("Find Profile")
        .raw_str("<input type=\"text\" name=\"username\" placeholder=\"Username (e.g., alice001)\" />\n")
        .form_link("Search", "search_profile")
        .newline()
        .newline();

    // Show viewer's profile status
    if let Some(viewer_addr) = viewer {
        let profile: Option<Profile> = env
            .storage()
            .persistent()
            .get(&ProfileKey::Profile(viewer_addr.clone()));

        match profile {
            Some(p) if p.is_active() => {
                md = md
                    .h2("Your Profile")
                    .text("Username: **@")
                    .raw(p.username.clone())
                    .text("**")
                    .newline()
                    .text("Display Name: ")
                    .text_string(&p.display_name)
                    .newline()
                    .newline()
                    .render_link("Edit Profile", "/edit")
                    .text(" | ")
                    .raw_str("[View Profile](render:/u/")
                    .raw(p.username)
                    .raw_str(")");
            }
            _ => {
                md = md
                    .h2("Get Started")
                    .paragraph("You don't have a profile yet.")
                    .render_link("Register Now", "/register");
            }
        }
    } else {
        md = md
            .h2("Get Started")
            .paragraph("Connect your wallet to register a profile.")
            .render_link("Register", "/register");
    }

    md.build()
}

/// Render the registration form.
fn render_register_form(env: &Env, viewer: &Option<Address>) -> Bytes {
    let mut md = MarkdownBuilder::new(env)
        .h1("Register Profile")
        .render_link("Back to Home", "/")
        .newline()
        .newline();

    if viewer.is_none() {
        return md
            .warning("Please connect your wallet to register.")
            .build();
    }

    let viewer_addr = viewer.as_ref().unwrap();

    // Check if already registered
    let existing: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(viewer_addr.clone()));

    if let Some(p) = existing {
        if p.is_active() {
            return md
                .warning("You already have a profile.")
                .raw_str("[View your profile](render:/u/")
                .raw(p.username)
                .raw_str(")")
                .build();
        }
    }

    md = md
        .h2("Username Requirements")
        .paragraph("Your username must:")
        .raw_str("- Be 6-17 characters long\n")
        .raw_str("- Start with 3+ lowercase letters\n")
        .raw_str("- End with 3 digits\n")
        .raw_str("- Only contain lowercase letters, digits, and underscores\n")
        .raw_str("\n**Examples:** `alice001`, `bob_smith123`, `crypto_fan_999`\n\n")
        .h2("Registration Form")
        .raw_str("<input type=\"text\" name=\"username\" placeholder=\"Username (e.g., alice001)\" required />\n")
        .raw_str("<input type=\"text\" name=\"display_name\" placeholder=\"Display Name\" required />\n")
        .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
        .raw(address_to_bytes(env, viewer_addr))
        .raw_str("\" />\n")
        .form_link("Register", "register")
        .newline();

    // Show fee if applicable
    let fee: i128 = env
        .storage()
        .instance()
        .get(&ProfileKey::RegistrationFee)
        .unwrap_or(0);

    if fee > 0 {
        md = md
            .newline()
            .text("**Registration fee:** ")
            .raw(i128_to_bytes(env, fee))
            .text(" stroops");
    }

    md.build()
}

/// Render the profile edit form.
fn render_edit_form(env: &Env, viewer: &Option<Address>) -> Bytes {
    let mut md = MarkdownBuilder::new(env)
        .h1("Edit Profile")
        .render_link("Back to Home", "/")
        .newline()
        .newline();

    if viewer.is_none() {
        return md
            .warning("Please connect your wallet.")
            .build();
    }

    let viewer_addr = viewer.as_ref().unwrap();

    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(viewer_addr.clone()));

    match profile {
        None => md.warning("No profile found. Please register first.").build(),
        Some(p) if !p.is_active() => md.warning("Your profile has been deleted.").build(),
        Some(p) => {
            // Display name edit
            md = md
                .h2("Display Name")
                .raw_str("<input type=\"text\" name=\"display_name\" value=\"")
                .text_string(&p.display_name)
                .raw_str("\" />\n")
                .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                .raw(address_to_bytes(env, viewer_addr))
                .raw_str("\" />\n")
                .form_link("Update Display Name", "set_display_name")
                .hr();

            // Standard fields
            let fields = [
                ("bio", "Bio", "Tell us about yourself..."),
                ("avatar", "Avatar URL", "https://..."),
                ("homepage", "Homepage", "https://..."),
                ("location", "Location", "City, Country"),
                ("github", "GitHub", "username"),
                ("twitter", "Twitter", "@handle"),
            ];

            for (field_name, label, placeholder) in fields.iter() {
                let current: Option<FieldValue> = env
                    .storage()
                    .persistent()
                    .get(&ProfileKey::Field(viewer_addr.clone(), Symbol::new(env, field_name)));

                md = md.h3(label);

                // Build input with current value if exists
                if let Some(FieldValue::StringField(ref v)) = current {
                    md = md
                        .raw_str("<input type=\"text\" name=\"value\" value=\"")
                        .text_string(v)
                        .raw_str("\" placeholder=\"")
                        .text(placeholder)
                        .raw_str("\" />\n");
                } else {
                    md = md
                        .raw_str("<input type=\"text\" name=\"value\" placeholder=\"")
                        .text(placeholder)
                        .raw_str("\" />\n");
                }

                md = md
                    .raw_str("<input type=\"hidden\" name=\"field\" value=\"")
                    .text(field_name)
                    .raw_str("\" />\n")
                    .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                    .raw(address_to_bytes(env, viewer_addr))
                    .raw_str("\" />\n")
                    .form_link("Update", "set_string_field")
                    .newline();
            }

            md = md
                .hr()
                .h2("Danger Zone")
                .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                .raw(address_to_bytes(env, viewer_addr))
                .raw_str("\" />\n")
                .form_link("Delete Profile", "delete_profile");

            md.build()
        }
    }
}

/// Render profile by username.
fn render_profile_by_username(env: &Env, username: &Bytes, viewer: &Option<Address>) -> Bytes {
    // Look up address from username
    let address: Option<Address> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Username(username.clone()));

    match address {
        Some(addr) => render_full_profile(env, &addr, viewer),
        None => {
            MarkdownBuilder::new(env)
                .h1("Profile Not Found")
                .paragraph("No profile found with that username.")
                .render_link("Back to Home", "/")
                .build()
        }
    }
}

/// Render profile by address string.
fn render_profile_by_address_str(env: &Env, _viewer: &Option<Address>) -> Bytes {
    MarkdownBuilder::new(env)
        .h1("Profile Lookup by Address")
        .paragraph("Address lookup requires a valid Stellar address format.")
        .render_link("Back to Home", "/")
        .build()
}

/// Render a full profile page.
fn render_full_profile(env: &Env, address: &Address, viewer: &Option<Address>) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        None => {
            MarkdownBuilder::new(env)
                .h1("Profile Not Found")
                .render_link("Back to Home", "/")
                .build()
        }
        Some(p) if !p.is_active() => {
            MarkdownBuilder::new(env)
                .h1("Profile Deleted")
                .paragraph("This profile has been deleted.")
                .render_link("Back to Home", "/")
                .build()
        }
        Some(p) => {
            let mut md = MarkdownBuilder::new(env)
                .render_link("Back to Home", "/")
                .newline()
                .newline();

            // Avatar if present
            if let Some(FieldValue::StringField(avatar)) = env
                .storage()
                .persistent()
                .get(&ProfileKey::Field(address.clone(), Symbol::new(env, "avatar")))
            {
                md = md
                    .raw_str("<img class=\"profile-avatar\" src=\"")
                    .text_string(&avatar)
                    .raw_str("\" alt=\"Avatar\" style=\"width: 100px; border-radius: 50%;\" />\n\n");
            }

            // Name and username
            md = md
                .raw_str("# ")
                .text_string(&p.display_name)
                .raw_str("\n\n")
                .text("**@")
                .raw(p.username.clone())
                .text("**")
                .newline()
                .newline();

            // Bio if present
            if let Some(FieldValue::StringField(bio)) = env
                .storage()
                .persistent()
                .get(&ProfileKey::Field(address.clone(), Symbol::new(env, "bio")))
            {
                md = md.text_string(&bio).raw_str("\n\n");
            }

            // Other fields
            let fields = [
                ("location", "Location"),
                ("homepage", "Website"),
                ("github", "GitHub"),
                ("twitter", "Twitter"),
            ];

            for (field_name, label) in fields.iter() {
                if let Some(FieldValue::StringField(value)) = env
                    .storage()
                    .persistent()
                    .get(&ProfileKey::Field(address.clone(), Symbol::new(env, field_name)))
                {
                    md = md
                        .text("**")
                        .text(label)
                        .text(":** ")
                        .text_string(&value)
                        .newline();
                }
            }

            // Show edit link if viewer is owner
            if let Some(viewer_addr) = viewer {
                if *viewer_addr == p.owner {
                    md = md
                        .newline()
                        .hr()
                        .render_link("Edit Profile", "/edit");
                }
            }

            md.build()
        }
    }
}

/// Render help page.
fn render_help(env: &Env) -> Bytes {
    MarkdownBuilder::new(env)
        .h1("Help")
        .render_link("Back to Home", "/")
        .newline()
        .newline()
        .h2("What is this?")
        .paragraph("This is a global user profile system for Soroban blockchain applications.")
        .paragraph("Your profile is stored on the Stellar blockchain and can be used by any compatible application.")
        .h2("Username Format")
        .paragraph("Usernames must follow this format:")
        .raw_str("- 6-17 characters\n")
        .raw_str("- Start with 3+ lowercase letters\n")
        .raw_str("- End with 3 digits\n")
        .raw_str("- Only lowercase letters, digits, underscores\n\n")
        .paragraph("Examples: `alice001`, `bob_smith123`, `crypto_fan_999`")
        .h2("Standard Fields")
        .paragraph("You can set these profile fields:")
        .raw_str("- **bio** - Your biography\n")
        .raw_str("- **avatar** - Avatar image URL\n")
        .raw_str("- **homepage** - Your website\n")
        .raw_str("- **location** - Where you're based\n")
        .raw_str("- **github** - GitHub username\n")
        .raw_str("- **twitter** - Twitter handle\n")
        .build()
}

/// Render a profile card for embedding in other contracts.
pub fn render_profile_card(env: &Env, address: &Address) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        None => {
            // Anonymous card
            MarkdownBuilder::new(env)
                .raw_str("<div class=\"profile-card profile-card-anonymous\">")
                .raw_str("<span class=\"profile-address\">")
                .raw(truncate_address_bytes(env, address))
                .raw_str("</span>")
                .raw_str("</div>")
                .build()
        }
        Some(p) if !p.is_active() => {
            MarkdownBuilder::new(env)
                .raw_str("<div class=\"profile-card profile-card-deleted\">")
                .raw_str("<span class=\"profile-deleted\">[deleted]</span>")
                .raw_str("</div>")
                .build()
        }
        Some(p) => {
            let mut md = MarkdownBuilder::new(env)
                .raw_str("<div class=\"profile-card\">");

            // Avatar if present
            if let Some(FieldValue::StringField(avatar)) = env
                .storage()
                .persistent()
                .get(&ProfileKey::Field(address.clone(), Symbol::new(env, "avatar")))
            {
                md = md
                    .raw_str("<img class=\"profile-avatar\" src=\"")
                    .text_string(&avatar)
                    .raw_str("\" />");
            }

            // Info
            md = md
                .raw_str("<div class=\"profile-info\">")
                .raw_str("<span class=\"profile-display-name\">")
                .text_string(&p.display_name)
                .raw_str("</span>")
                .raw_str("<span class=\"profile-username\">@")
                .raw(p.username.clone())
                .raw_str("</span>")
                .raw_str("</div>");

            // Link to profile (uses @profile alias for cross-contract navigation)
            md = md
                .raw_str("<a href=\"render:@profile:/u/")
                .raw(p.username)
                .raw_str("\">View Profile</a>")
                .raw_str("</div>");

            md.build()
        }
    }
}

/// Render a compact profile card (for author attribution).
pub fn render_profile_card_compact(env: &Env, address: &Address) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        Some(p) if p.is_active() => {
            // Uses @profile alias for cross-contract navigation
            MarkdownBuilder::new(env)
                .raw_str("<span class=\"profile-compact\">")
                .raw_str("<a href=\"render:@profile:/u/")
                .raw(p.username.clone())
                .raw_str("\">@")
                .raw(p.username)
                .raw_str("</a>")
                .raw_str("</span>")
                .build()
        }
        _ => {
            MarkdownBuilder::new(env)
                .raw_str("<span class=\"profile-compact profile-anonymous\">")
                .raw(truncate_address_bytes(env, address))
                .raw_str("</span>")
                .build()
        }
    }
}

/// Render just the username (or truncated address).
pub fn render_username(env: &Env, address: &Address) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        Some(p) if p.is_active() => {
            MarkdownBuilder::new(env)
                .text("@")
                .raw(p.username)
                .build()
        }
        _ => {
            MarkdownBuilder::new(env)
                .raw(truncate_address_bytes(env, address))
                .build()
        }
    }
}

// ========== Helper Functions ==========

/// Convert Address to Bytes for display.
fn address_to_bytes(env: &Env, address: &Address) -> Bytes {
    // Convert Address to String, then to Bytes
    let addr_string = address.to_string();
    soroban_render_sdk::bytes::string_to_bytes(env, &addr_string)
}

/// Truncate an address for display (returns Bytes).
fn truncate_address_bytes(env: &Env, address: &Address) -> Bytes {
    let full_bytes = address_to_bytes(env, address);
    let len = full_bytes.len();

    if len <= 12 {
        // Short enough, return as-is
        return full_bytes;
    }

    // Truncate: GXXX...XXXX
    let mut result = Bytes::new(env);

    // First 4 characters
    for i in 0..4 {
        if let Some(c) = full_bytes.get(i) {
            result.push_back(c);
        }
    }

    // Ellipsis
    result.push_back(b'.');
    result.push_back(b'.');
    result.push_back(b'.');

    // Last 4 characters
    for i in (len - 4)..len {
        if let Some(c) = full_bytes.get(i) {
            result.push_back(c);
        }
    }

    result
}

/// Convert u64 to Bytes.
fn u64_to_bytes(env: &Env, n: u64) -> Bytes {
    if n == 0 {
        return Bytes::from_slice(env, b"0");
    }

    let mut buffer = [0u8; 20];
    let mut idx = 20;
    let mut num = n;

    while num > 0 {
        idx -= 1;
        buffer[idx] = b'0' + (num % 10) as u8;
        num /= 10;
    }

    Bytes::from_slice(env, &buffer[idx..])
}

/// Convert i128 to Bytes.
fn i128_to_bytes(env: &Env, n: i128) -> Bytes {
    if n == 0 {
        return Bytes::from_slice(env, b"0");
    }

    let is_negative = n < 0;
    let mut num = if is_negative { -n } else { n } as u128;
    let mut buffer = [0u8; 40];
    let mut idx = 40;

    while num > 0 {
        idx -= 1;
        buffer[idx] = b'0' + (num % 10) as u8;
        num /= 10;
    }

    if is_negative {
        idx -= 1;
        buffer[idx] = b'-';
    }

    Bytes::from_slice(env, &buffer[idx..])
}
