#![warn(clippy::all, clippy::pedantic)]
use wasm_bindgen::prelude::*;
use web_sys::{window, Event, HtmlElement, HtmlInputElement, MediaQueryList};

// Import crypto module for browser-side decryption
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, ParamsBuilder, Version,
};
use base64::prelude::*;

/// Theme preference options: light, dark, or auto (follow system)
#[derive(Debug, PartialEq, Clone, Copy)]
enum ThemePreference {
    Light,
    Dark,
    Auto,
}

impl ThemePreference {
    fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::Auto, // Default to auto (includes "auto" and unknown values)
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "auto",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Light => "✸",
            Self::Dark => "☽",
            Self::Auto => "◐",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Auto,
            Self::Auto => Self::Light,
        }
    }
}

/// Initialize the web assembly module and synchronize the stored theme on load.
///
/// # Errors
/// Returns an error if the DOM or `localStorage` cannot be accessed, or if the theme
/// attribute fails to update.
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Initialize theme on page load
    init_theme()?;

    // Initialize locked entry decryption UI (if present)
    init_locked_entry()?;

    Ok(())
}

/// Toggle the theme preference through the cycle: light → dark → auto → light
///
/// # Errors
/// Returns an error when the DOM, its elements, or `localStorage` cannot be accessed
/// or updated.
#[wasm_bindgen]
pub fn toggle_theme() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let local_storage = window.local_storage()?.ok_or("no localStorage")?;

    // Get current preference
    let current_pref = local_storage
        .get_item("theme-preference")?
        .map_or(ThemePreference::Auto, |s| ThemePreference::from_str(&s));

    // Cycle to next preference
    let new_pref = current_pref.next();

    // Save new preference
    local_storage.set_item("theme-preference", new_pref.as_str())?;

    // Apply the theme
    apply_theme(new_pref)?;

    Ok(())
}

/// Apply the specified theme preference to the DOM
///
/// # Errors
/// Returns an error when the DOM or its elements cannot be accessed or updated.
fn apply_theme(preference: ThemePreference) -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Determine actual theme to apply
    let actual_theme = match preference {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
        ThemePreference::Auto => {
            // Detect system preference
            if is_system_dark_mode()? {
                "dark"
            } else {
                "light"
            }
        }
    };

    // Update DOM
    document
        .document_element()
        .ok_or("no document element")?
        .set_attribute("data-theme", actual_theme)?;

    // Update icon to show preference (not actual theme)
    let icon_element = document
        .get_element_by_id("theme-icon")
        .ok_or("no theme-icon element")?;

    icon_element.set_text_content(Some(preference.icon()));

    Ok(())
}

/// Check if the system prefers dark mode
///
/// # Errors
/// Returns an error if the media query cannot be accessed.
fn is_system_dark_mode() -> Result<bool, JsValue> {
    let window = window().ok_or("no window")?;
    let media_query = window
        .match_media("(prefers-color-scheme: dark)")?
        .ok_or("no media query")?;
    Ok(media_query.matches())
}

/// Set up a listener for system theme changes (only when preference is "auto")
///
/// # Errors
/// Returns an error if the event listener cannot be set up.
fn setup_system_theme_listener() -> Result<(), JsValue> {
    let win = window().ok_or("no window")?;

    let media_query: MediaQueryList = win
        .match_media("(prefers-color-scheme: dark)")?
        .ok_or("no media query")?;

    let closure = Closure::wrap(Box::new(move || {
        // Check if we're in auto mode
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(pref_str)) = storage.get_item("theme-preference") {
                    let preference = ThemePreference::from_str(&pref_str);
                    if preference == ThemePreference::Auto {
                        // Reapply theme to pick up system change
                        let _ = apply_theme(preference);
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);

    // Add event listener for changes
    media_query.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn init_theme() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let local_storage = window.local_storage()?.ok_or("no localStorage")?;

    // Get saved preference or default to auto
    let preference = local_storage
        .get_item("theme-preference")?
        .map_or(ThemePreference::Auto, |s| ThemePreference::from_str(&s));

    // Apply theme based on preference
    apply_theme(preference)?;

    // Set up system theme change listener
    setup_system_theme_listener()?;

    // Add click event listener to toggle button
    let toggle_button = document
        .get_element_by_id("theme-toggle")
        .ok_or("no theme-toggle element")?
        .dyn_into::<HtmlElement>()?;

    let closure = Closure::wrap(Box::new(move || {
        let _ = toggle_theme();
    }) as Box<dyn FnMut()>);

    toggle_button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    Ok(())
}

// ============================================================================
// Locked Entry Decryption (Browser-side)
// ============================================================================

// Argon2id parameters (must match src/crypto.rs)
const ARGON2_MEMORY: u32 = 65536; // 64 MB
const ARGON2_TIME: u32 = 3; // iterations
const ARGON2_PARALLELISM: u32 = 4; // threads

/// Initialize locked entry UI if present on the page
///
/// # Errors
/// Returns an error if DOM elements cannot be accessed (ignored if no locked entry)
fn init_locked_entry() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Check if this page has a locked entry
    let Some(locked_entry) = document.get_element_by_id("locked-entry-container") else {
        return Ok(()); // No locked entry on this page
    };

    let passphrase_input = document
        .get_element_by_id("passphrase-input")
        .ok_or("no passphrase-input")?
        .dyn_into::<HtmlInputElement>()?;

    let decrypt_button = document
        .get_element_by_id("decrypt-button")
        .ok_or("no decrypt-button")?
        .dyn_into::<HtmlElement>()?;

    // Get encrypted data from data attribute
    let encrypted_b64 = locked_entry
        .get_attribute("data-encrypted")
        .ok_or("no data-encrypted attribute")?;

    // Clone for closure
    let encrypted_b64_clone = encrypted_b64.clone();
    let passphrase_input_clone = passphrase_input.clone();

    // Decrypt button click handler
    let decrypt_closure = Closure::wrap(Box::new(move |_event: Event| {
        let _ = handle_decrypt(&encrypted_b64_clone, &passphrase_input_clone);
    }) as Box<dyn FnMut(Event)>);

    decrypt_button.set_onclick(Some(decrypt_closure.as_ref().unchecked_ref()));
    decrypt_closure.forget();

    // Also trigger on Enter key in input
    let encrypted_b64_clone2 = encrypted_b64.clone();
    let passphrase_input_clone2 = passphrase_input.clone();
    let enter_closure = Closure::wrap(Box::new(move |event: Event| {
        // Check if Enter key was pressed
        if let Some(keyboard_event) = event.dyn_ref::<web_sys::KeyboardEvent>() {
            if keyboard_event.key() == "Enter" {
                let _ = handle_decrypt(&encrypted_b64_clone2, &passphrase_input_clone2);
            }
        }
    }) as Box<dyn FnMut(Event)>);

    passphrase_input
        .add_event_listener_with_callback("keydown", enter_closure.as_ref().unchecked_ref())?;
    enter_closure.forget();

    Ok(())
}

/// Handle decrypt button click
fn handle_decrypt(encrypted_b64: &str, passphrase_input: &HtmlInputElement) -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Get passphrase from input
    let passphrase = passphrase_input.value();
    if passphrase.is_empty() {
        show_error("Please enter a passphrase")?;
        return Ok(());
    }

    // Show decrypting status
    show_status("Decrypting...")?;

    // Decode base64
    let Ok(encrypted_bytes) = BASE64_STANDARD.decode(encrypted_b64) else {
        show_error("Invalid encrypted data format")?;
        return Ok(());
    };

    // Decrypt
    match decrypt_content(&encrypted_bytes, &passphrase) {
        Ok(plaintext) => {
            // Parse markdown to HTML (simple conversion for now)
            let html = markdown_to_html(&plaintext);

            // Display decrypted content
            let content_div = document
                .get_element_by_id("decrypted-content")
                .ok_or("no decrypted-content")?;
            content_div.set_inner_html(&html);

            // Remove blur from preview with transition
            if let Some(locked_preview) = document.get_element_by_id("locked-preview") {
                locked_preview.set_class_name("locked-preview");
            }

            // Hide unlock overlay with fade
            if let Some(unlock_overlay) = document.get_element_by_id("unlock-overlay") {
                unlock_overlay.set_class_name("unlock-overlay hidden");
            }

            // Hide blurred preview after transition (500ms)
            if let Some(locked_preview) = document.get_element_by_id("locked-preview") {
                let preview_clone = locked_preview.clone();
                let closure = Closure::once(Box::new(move || {
                    preview_clone.set_class_name("hidden");
                }) as Box<dyn FnOnce()>);

                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        500,
                    )?;
                closure.forget();
            }

            // Show decrypted content
            content_div.set_class_name("decrypted-content");

            // Clear passphrase input (security)
            passphrase_input.set_value("");

            Ok(())
        }
        Err(e) => {
            show_error(&format!("Decryption failed: {e}"))?;
            Ok(())
        }
    }
}

/// Decrypt encrypted content using AES-256-GCM + Argon2id
fn decrypt_content(ciphertext: &[u8], passphrase: &str) -> Result<String, String> {
    // Find delimiter position
    let delimiter_pos = ciphertext
        .iter()
        .position(|&b| b == b'|')
        .ok_or("Invalid ciphertext format: delimiter not found")?;

    // Extract salt string
    let salt_bytes = &ciphertext[..delimiter_pos];
    let salt_str = std::str::from_utf8(salt_bytes).map_err(|_| "Salt is not valid UTF-8")?;
    let salt = SaltString::from_b64(salt_str).map_err(|e| format!("Failed to parse salt: {e}"))?;

    // Extract nonce (12 bytes after delimiter)
    let nonce_start = delimiter_pos + 1;
    let nonce_end = nonce_start + 12;
    if ciphertext.len() < nonce_end {
        return Err("Ciphertext too short for nonce".to_string());
    }
    let nonce_bytes = &ciphertext[nonce_start..nonce_end];
    let nonce = Nonce::from_slice(nonce_bytes);

    // Extract ciphertext data
    let ciphertext_data = &ciphertext[nonce_end..];

    // Derive key using Argon2id
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY)
            .t_cost(ARGON2_TIME)
            .p_cost(ARGON2_PARALLELISM)
            .output_len(32)
            .build()
            .map_err(|e| format!("Failed to build Argon2 parameters: {e}"))?,
    );

    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| format!("Failed to derive key with Argon2id: {e}"))?;

    let key_bytes = password_hash.hash.ok_or("Argon2 hash output is missing")?;

    // Create AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(key_bytes.as_bytes())
        .map_err(|_| "Failed to create AES-256-GCM cipher")?;

    // Decrypt and verify
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext_data)
        .map_err(|_| "Decryption failed: incorrect passphrase or corrupted data")?;

    // Convert to UTF-8 string
    let plaintext =
        String::from_utf8(plaintext_bytes).map_err(|_| "Decrypted content is not valid UTF-8")?;

    Ok(plaintext)
}

/// Show error message in UI
fn show_error(message: &str) -> Result<(), JsValue> {
    let document = window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    // Hide status if present
    if let Some(status) = document.get_element_by_id("decrypt-status") {
        status.set_class_name("hidden");
    }

    let error_div = document
        .get_element_by_id("error-message")
        .ok_or("no error-message")?;
    error_div.set_text_content(Some(message));
    error_div.set_class_name("error-message");
    Ok(())
}

/// Show status message in UI
fn show_status(message: &str) -> Result<(), JsValue> {
    let document = window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    // Hide error if present
    if let Some(error) = document.get_element_by_id("error-message") {
        error.set_class_name("hidden");
    }

    let status_div = document
        .get_element_by_id("decrypt-status")
        .ok_or("no decrypt-status")?;
    status_div.set_text_content(Some(message));
    status_div.set_class_name("decrypt-status");
    Ok(())
}

/// Simple markdown to HTML conversion (basic implementation)
fn markdown_to_html(markdown: &str) -> String {
    // For now, use a very basic conversion
    // In production, you'd want to use a proper markdown parser
    let mut html = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                html.push_str("<pre><code>");
            } else {
                html.push_str("</code></pre>\n");
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
        } else if let Some(stripped) = line.strip_prefix("### ") {
            html.push_str("<h3>");
            html.push_str(&process_inline_html(stripped));
            html.push_str("</h3>\n");
        } else if let Some(stripped) = line.strip_prefix("## ") {
            html.push_str("<h2>");
            html.push_str(&process_inline_html(stripped));
            html.push_str("</h2>\n");
        } else if let Some(stripped) = line.strip_prefix("# ") {
            html.push_str("<h1>");
            html.push_str(&process_inline_html(stripped));
            html.push_str("</h1>\n");
        } else if line.is_empty() {
            html.push_str("<br>\n");
        } else {
            html.push_str("<p>");
            html.push_str(&process_inline_html(line));
            html.push_str("</p>\n");
        }
    }

    html
}

/// Process inline HTML - allows certain safe HTML tags while escaping others
fn process_inline_html(s: &str) -> String {
    // Allow <span> tags with class attributes (for timestamps, etc.)
    // This is a simple implementation - in production use a proper HTML sanitizer
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Try to parse a tag
            let mut tag = String::from("<");

            // Check if it's a closing tag
            if chars.peek() == Some(&'/') {
                tag.push(chars.next().unwrap());
            }

            // Get tag name
            let mut tag_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '>' || next_ch == ' ' {
                    break;
                }
                tag_name.push(next_ch);
                tag.push(next_ch);
                chars.next();
            }

            // Collect rest of tag
            while let Some(&next_ch) = chars.peek() {
                tag.push(next_ch);
                chars.next();
                if next_ch == '>' {
                    break;
                }
            }

            // Allow span tags, escape others
            if tag_name == "span" || tag_name == "strong" || tag_name == "em" || tag_name == "code"
            {
                result.push_str(&tag);
            } else {
                // Escape the tag
                result.push_str(&html_escape(&tag));
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests `ThemePreference::from_str` for valid theme strings.
    /// Verifies correct enum variant parsing from string literals.
    #[test]
    fn test_theme_preference_from_str_valid() {
        assert_eq!(ThemePreference::from_str("light"), ThemePreference::Light);
        assert_eq!(ThemePreference::from_str("dark"), ThemePreference::Dark);
        assert_eq!(ThemePreference::from_str("auto"), ThemePreference::Auto);
    }

    /// Tests `ThemePreference::from_str` fallback for invalid strings.
    /// Verifies default to Auto for unrecognized values.
    #[test]
    fn test_theme_preference_from_str_invalid() {
        assert_eq!(ThemePreference::from_str(""), ThemePreference::Auto);
        assert_eq!(ThemePreference::from_str("invalid"), ThemePreference::Auto);
        assert_eq!(ThemePreference::from_str("LIGHT"), ThemePreference::Auto);
    }

    /// Tests `ThemePreference::as_str` conversion.
    /// Verifies correct string representation for each theme variant.
    #[test]
    fn test_theme_preference_as_str() {
        assert_eq!(ThemePreference::Light.as_str(), "light");
        assert_eq!(ThemePreference::Dark.as_str(), "dark");
        assert_eq!(ThemePreference::Auto.as_str(), "auto");
    }

    /// Tests bidirectional string conversion.
    /// Verifies `from_str` and `as_str` are inverse operations for valid inputs.
    #[test]
    fn test_theme_preference_round_trip() {
        let themes = [
            ThemePreference::Light,
            ThemePreference::Dark,
            ThemePreference::Auto,
        ];

        for theme in themes {
            let string = theme.as_str();
            let parsed = ThemePreference::from_str(string);
            assert_eq!(parsed, theme);
        }
    }

    /// Tests `ThemePreference::icon` for all variants.
    /// Verifies correct icon character for each theme state.
    #[test]
    fn test_theme_preference_icon() {
        assert_eq!(ThemePreference::Light.icon(), "✸");
        assert_eq!(ThemePreference::Dark.icon(), "☽");
        assert_eq!(ThemePreference::Auto.icon(), "◐");
    }

    /// Tests `ThemePreference::next` cycling behavior.
    /// Verifies Light → Dark → Auto → Light cycle.
    #[test]
    fn test_theme_preference_next() {
        assert_eq!(ThemePreference::Light.next(), ThemePreference::Dark);
        assert_eq!(ThemePreference::Dark.next(), ThemePreference::Auto);
        assert_eq!(ThemePreference::Auto.next(), ThemePreference::Light);
    }

    /// Tests complete theme preference cycle.
    /// Verifies three `next()` calls return to starting state.
    #[test]
    fn test_theme_preference_full_cycle() {
        let start = ThemePreference::Light;
        let after_one = start.next();
        let after_two = after_one.next();
        let after_three = after_two.next();
        assert_eq!(after_three, start);
    }

    /// Tests `ThemePreference` Debug trait implementation.
    /// Verifies debug formatting produces expected output.
    #[test]
    fn test_theme_preference_debug() {
        assert_eq!(format!("{:?}", ThemePreference::Light), "Light");
        assert_eq!(format!("{:?}", ThemePreference::Dark), "Dark");
        assert_eq!(format!("{:?}", ThemePreference::Auto), "Auto");
    }

    /// Tests `ThemePreference` `PartialEq` implementation.
    /// Verifies equality comparison works correctly.
    #[test]
    fn test_theme_preference_equality() {
        assert_eq!(ThemePreference::Light, ThemePreference::Light);
        assert_eq!(ThemePreference::Dark, ThemePreference::Dark);
        assert_eq!(ThemePreference::Auto, ThemePreference::Auto);

        assert_ne!(ThemePreference::Light, ThemePreference::Dark);
        assert_ne!(ThemePreference::Dark, ThemePreference::Auto);
        assert_ne!(ThemePreference::Auto, ThemePreference::Light);
    }

    /// Tests `ThemePreference` Clone trait implementation.
    /// Verifies cloning produces equal values.
    #[test]
    fn test_theme_preference_clone() {
        let original = ThemePreference::Light;
        let cloned = original;
        assert_eq!(original, cloned);
    }
}
