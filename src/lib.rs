use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Initialize theme on page load
    init_theme()?;
    Ok(())
}

#[wasm_bindgen]
pub fn toggle_theme() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let local_storage = window.local_storage()?.ok_or("no localStorage")?;

    // Get current theme
    let current_theme = document
        .document_element()
        .ok_or("no document element")?
        .get_attribute("data-theme")
        .unwrap_or_else(|| "light".to_string());

    // Toggle theme
    let new_theme = if current_theme == "dark" {
        "light"
    } else {
        "dark"
    };

    // Update DOM
    document
        .document_element()
        .ok_or("no document element")?
        .set_attribute("data-theme", new_theme)?;

    // Update icon
    let icon_element = document
        .get_element_by_id("theme-icon")
        .ok_or("no theme-icon element")?;

    icon_element.set_text_content(Some(if new_theme == "dark" { "☽" } else { "✸" }));

    // Save to localStorage
    local_storage.set_item("theme", new_theme)?;

    Ok(())
}

fn init_theme() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let local_storage = window.local_storage()?.ok_or("no localStorage")?;

    // Get saved theme or default to light
    let theme = local_storage
        .get_item("theme")?
        .unwrap_or_else(|| "light".to_string());

    // Apply theme
    if theme == "dark" {
        document
            .document_element()
            .ok_or("no document element")?
            .set_attribute("data-theme", "dark")?;

        let icon_element = document
            .get_element_by_id("theme-icon")
            .ok_or("no theme-icon element")?;
        icon_element.set_text_content(Some("☽"));
    }

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
