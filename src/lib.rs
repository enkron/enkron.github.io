#![warn(clippy::all, clippy::pedantic)]
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement, MediaQueryList};

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
        assert_eq!(
            ThemePreference::from_str("invalid"),
            ThemePreference::Auto
        );
        assert_eq!(
            ThemePreference::from_str("LIGHT"),
            ThemePreference::Auto
        );
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
