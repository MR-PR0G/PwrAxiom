pub mod controls;
pub mod dashboard;
pub mod settings;
pub mod spotlight;

use gtk4::gdk::Display;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

pub fn load_css() {
    let provider = CssProvider::new();
    let css_data = include_str!("style.css");

    provider.load_from_data(css_data);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    dashboard::load_saved_theme();
}