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
        // Handle profile with return path: /u/{username}/from/{return_path}
        .or_handle(b"/u/{username}/from/*", |req| {
            let username = req.get_var(b"username").unwrap_or_else(|| Bytes::new(env));
            let return_path = req.get_wildcard().unwrap_or_else(|| Bytes::new(env));
            render_profile_by_username_with_return(env, &username, &viewer, Some(return_path))
        })
        .or_handle(b"/u/*", |req| {
            let username = req.get_wildcard().unwrap_or_else(|| Bytes::new(env));
            render_profile_by_username_with_return(env, &username, &viewer, None)
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
        .raw_str("<input type=\"hidden\" name=\"_redirect\" value=\"/edit\" />\n")
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
            // Show timestamps
            md = md
                .raw_str("<div class=\"profile-timestamps\">")
                .raw_str("<small>")
                .text("Created: ")
                .raw(format_timestamp(env, p.created_at))
                .text(" · Updated: ")
                .raw(format_timestamp(env, p.updated_at))
                .raw_str("</small>")
                .raw_str("</div>")
                .newline()
                .newline();

            // Display name edit
            md = md
                .h2("Display Name")
                .raw_str("<div data-form>\n")
                .raw_str("<input type=\"text\" name=\"display_name\" value=\"")
                .text_string(&p.display_name)
                .raw_str("\" />\n")
                .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                .raw(address_to_bytes(env, viewer_addr))
                .raw_str("\" />\n")
                .form_link("Update Display Name", "set_display_name")
                .raw_str("</div>\n")
                .newline() // Blank line needed for markdown parser before HR
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

                // Build input with current value if exists, wrapped in form boundary
                // Input order must match contract signature: field, value, caller
                md = md
                    .raw_str("<div data-form>\n")
                    .raw_str("<input type=\"hidden\" name=\"field\" value=\"")
                    .text(field_name)
                    .raw_str("\" />\n");

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
                    .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                    .raw(address_to_bytes(env, viewer_addr))
                    .raw_str("\" />\n")
                    .form_link("Update", "set_string_field")
                    .raw_str("</div>\n")
                    .newline()
                    .newline(); // Blank line needed for markdown parser to recognize next h3
            }

            md = md
                .hr()
                .h2("Danger Zone")
                .raw_str("<div data-form>\n")
                .raw_str("<input type=\"hidden\" name=\"caller\" value=\"")
                .raw(address_to_bytes(env, viewer_addr))
                .raw_str("\" />\n")
                .form_link("Delete Profile", "delete_profile")
                .raw_str("</div>\n");

            md.build()
        }
    }
}

/// Render profile by username with optional return path.
fn render_profile_by_username_with_return(
    env: &Env,
    username: &Bytes,
    viewer: &Option<Address>,
    return_path: Option<Bytes>,
) -> Bytes {
    // Look up address from username
    let address: Option<Address> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Username(username.clone()));

    match address {
        Some(addr) => render_full_profile(env, &addr, viewer, return_path),
        None => {
            let mut md = MarkdownBuilder::new(env)
                .h1("Profile Not Found")
                .paragraph("No profile found with that username.");
            md = render_back_link(env, md, &return_path);
            md.build()
        }
    }
}

/// Render a "Go Back" or "Back to Home" link based on return path.
fn render_back_link<'a>(
    _env: &Env,
    md: MarkdownBuilder<'a>,
    return_path: &Option<Bytes>,
) -> MarkdownBuilder<'a> {
    match return_path {
        Some(path) if !path.is_empty() => {
            // Use the return path with "Go Back" text
            md.raw_str("[Go Back](render:/")
                .raw(path.clone())
                .raw_str(")")
        }
        _ => {
            // Default to profile home
            md.render_link("Back to Home", "/")
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

/// Render a full profile page with optional return path.
fn render_full_profile(
    env: &Env,
    address: &Address,
    viewer: &Option<Address>,
    return_path: Option<Bytes>,
) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        None => {
            let mut md = MarkdownBuilder::new(env).h1("Profile Not Found");
            md = render_back_link(env, md, &return_path);
            md.build()
        }
        Some(p) if !p.is_active() => {
            let mut md = MarkdownBuilder::new(env)
                .h1("Profile Deleted")
                .paragraph("This profile has been deleted.");
            md = render_back_link(env, md, &return_path);
            md.build()
        }
        Some(p) => {
            let mut md = MarkdownBuilder::new(env);
            md = render_back_link(env, md, &return_path);
            md = md.newline().newline();

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

            // Timestamps
            md = md
                .newline()
                .raw_str("<div class=\"profile-timestamps\">")
                .raw_str("<small>")
                .text("Created: ")
                .raw(format_timestamp(env, p.created_at))
                .text(" · Updated: ")
                .raw(format_timestamp(env, p.updated_at))
                .raw_str("</small>")
                .raw_str("</div>");

            // Show edit link if viewer is owner
            if let Some(viewer_addr) = viewer {
                if *viewer_addr == p.owner {
                    md = md
                        .newline()
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
    render_profile_card_compact_with_return(env, address, None)
}

/// Render a compact profile card with optional return path.
///
/// When `return_path` is provided, clicking the profile link will include
/// the return path so the "Go Back" button on the profile page returns
/// to the original location.
pub fn render_profile_card_compact_with_return(
    env: &Env,
    address: &Address,
    return_path: Option<Bytes>,
) -> Bytes {
    let profile: Option<Profile> = env
        .storage()
        .persistent()
        .get(&ProfileKey::Profile(address.clone()));

    match profile {
        Some(p) if p.is_active() => {
            // Uses @profile alias for cross-contract navigation
            let mut md = MarkdownBuilder::new(env)
                .raw_str("<span class=\"profile-compact\">")
                .raw_str("<a href=\"render:@profile:/u/")
                .raw(p.username.clone());

            // Add return path if provided
            if let Some(ref path) = return_path {
                if !path.is_empty() {
                    md = md.raw_str("/from/").raw(path.clone());
                }
            }

            md.raw_str("\">@")
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

/// Render a navigation link for embedding in other contracts' nav bars.
///
/// Returns:
/// - "@username" link to profile if viewer has a profile
/// - "Create Profile" link to registration if viewer has no profile
/// - "Create Profile" link if no viewer is connected
pub fn render_nav_link(env: &Env, viewer: &Option<Address>) -> Bytes {
    match viewer {
        None => {
            // Not connected - show Create Profile link
            MarkdownBuilder::new(env)
                .raw_str("<a href=\"render:@profile:/register\">Create Profile</a>")
                .build()
        }
        Some(addr) => {
            // Check if they have a profile
            let profile: Option<Profile> = env
                .storage()
                .persistent()
                .get(&ProfileKey::Profile(addr.clone()));

            match profile {
                Some(p) if p.is_active() => {
                    // Has profile - show @username linking to their profile
                    MarkdownBuilder::new(env)
                        .raw_str("<a href=\"render:@profile:/u/")
                        .raw(p.username.clone())
                        .raw_str("\">@")
                        .raw(p.username)
                        .raw_str("</a>")
                        .build()
                }
                _ => {
                    // No profile or deleted - show Create Profile link
                    MarkdownBuilder::new(env)
                        .raw_str("<a href=\"render:@profile:/register\">Create Profile</a>")
                        .build()
                }
            }
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

/// Format a Unix timestamp as a readable date string.
/// Returns format: "YYYY-MM-DD HH:MM:SS UTC"
fn format_timestamp(env: &Env, timestamp: u64) -> Bytes {
    // Handle legacy ledger sequence numbers (small values)
    // Unix timestamps for 2024+ are ~1700000000+
    if timestamp < 1_000_000_000 {
        // This is likely a ledger sequence, not a timestamp
        let mut result = Bytes::from_slice(env, b"Ledger ");
        result.append(&u64_to_bytes(env, timestamp));
        return result;
    }

    // Convert Unix timestamp to date components
    // Days since Unix epoch
    let total_seconds = timestamp;
    let total_minutes = total_seconds / 60;
    let total_hours = total_minutes / 60;
    let total_days = total_hours / 24;

    let seconds = (total_seconds % 60) as u8;
    let minutes = (total_minutes % 60) as u8;
    let hours = (total_hours % 24) as u8;

    // Calculate year, month, day from days since epoch (Jan 1, 1970)
    let (year, month, day) = days_to_date(total_days as i64);

    // Format: "YYYY-MM-DD HH:MM:SS UTC"
    let mut buffer = [0u8; 24];

    // Year (4 digits)
    buffer[0] = b'0' + ((year / 1000) % 10) as u8;
    buffer[1] = b'0' + ((year / 100) % 10) as u8;
    buffer[2] = b'0' + ((year / 10) % 10) as u8;
    buffer[3] = b'0' + (year % 10) as u8;
    buffer[4] = b'-';

    // Month (2 digits)
    buffer[5] = b'0' + ((month / 10) % 10) as u8;
    buffer[6] = b'0' + (month % 10) as u8;
    buffer[7] = b'-';

    // Day (2 digits)
    buffer[8] = b'0' + ((day / 10) % 10) as u8;
    buffer[9] = b'0' + (day % 10) as u8;
    buffer[10] = b' ';

    // Hours (2 digits)
    buffer[11] = b'0' + ((hours / 10) % 10);
    buffer[12] = b'0' + (hours % 10);
    buffer[13] = b':';

    // Minutes (2 digits)
    buffer[14] = b'0' + ((minutes / 10) % 10);
    buffer[15] = b'0' + (minutes % 10);
    buffer[16] = b':';

    // Seconds (2 digits)
    buffer[17] = b'0' + ((seconds / 10) % 10);
    buffer[18] = b'0' + (seconds % 10);

    // " UTC"
    buffer[19] = b' ';
    buffer[20] = b'U';
    buffer[21] = b'T';
    buffer[22] = b'C';

    Bytes::from_slice(env, &buffer[..23])
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days: i64) -> (i32, u8, u8) {
    // Algorithm based on Howard Hinnant's date algorithms
    // http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year
    let mp = (5 * doy + 2) / 153; // month index
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as i32, m as u8, d as u8)
}
