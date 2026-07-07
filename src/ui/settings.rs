use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, FlowBox, Label, Orientation, SelectionMode, Button, Window, CssProvider, ScrolledWindow, PolicyType
};
use glib::clone;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub type StdBox<T> = std::boxed::Box<T>;

pub const APP_NAME: &str = "Power Axiom";
pub const AUTHOR: &str = "Mr.Prog";
pub const PROJECT_DESCRIPTION: &str = "Hardware performance and power management utility for Linux built with Rust and GTK4.";
pub const GITHUB_URL: &str = "https://github.com/MR-PR0G/PwrAxiom";
pub const PROJECT_WEBSITE: &str = "https://github.com/MR-PR0G/PwrAxiom";

const GAP_SMALL: i32 = 8;
const GAP_LARGE: i32 = 24;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GitHubRelease {
    tag_name: String,
}

pub struct ThemeColor {
    pub name: &'static str,
    pub color: &'static str,
}

pub const THEME_COLORS: [ThemeColor; 12] = [
    ThemeColor { name: "Cyan Spark", color: "#00e5ff" },
    ThemeColor { name: "Emerald", color: "#00ff99" },
    ThemeColor { name: "Crimson Blaze", color: "#ff3366" },
    ThemeColor { name: "Amber", color: "#ffaa00" },
    ThemeColor { name: "Amethyst", color: "#b030ff" },
    ThemeColor { name: "Frost White", color: "#ffffff" },
    ThemeColor { name: "Neon Orchid", color: "#e056fd" },
    ThemeColor { name: "Coral Sunset", color: "#ff7675" },
    ThemeColor { name: "Ruby", color: "#ff003c" },
    ThemeColor { name: "Cobalt", color: "#0984e3" },
    ThemeColor { name: "Mint", color: "#2ed573" },
    ThemeColor { name: "Crimson", color: "#d63031" },
];

fn get_theme_config_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".pwraxiom_theme");
    path
}

fn save_theme_to_disk(color: &str) {
    let path = get_theme_config_path();
    let _ = std::fs::write(path, color);
}

fn load_theme_from_disk() -> String {
    let path = get_theme_config_path();
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "#00e5ff".to_string())
}

fn apply_theme(base_color: &str) {
    if let Some(display) = gtk4::gdk::Display::default() {
        let provider = CssProvider::new();
        let r_str = if base_color.len() >= 7 { &base_color[1..3] } else { "00" };
        let g_str = if base_color.len() >= 7 { &base_color[3..5] } else { "229" };
        let b_str = if base_color.len() >= 7 { &base_color[5..7] } else { "255" };

        let css = format!(
            ":root {{ --primary-color: {0}; --secondary-color: rgba({1}, {2}, {3}, 0.25); }}",
            base_color,
            u8::from_str_radix(r_str, 16).unwrap_or(0),
            u8::from_str_radix(g_str, 16).unwrap_or(229),
            u8::from_str_radix(b_str, 16).unwrap_or(255)
        );
        provider.load_from_data(&css);
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        save_theme_to_disk(base_color);
    }
}

fn create_row(label_text: &str, widget: &Button) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, GAP_SMALL);
    row.add_css_class("settings-row-layout");
    let lbl = Label::builder().label(label_text).halign(Align::Start).hexpand(true).build();
    row.append(&lbl);
    row.append(widget);
    row
}

fn build_card_container(title_text: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 12);
    container.add_css_class("theme-section-container");
    container.set_margin_start(35);
    container.set_margin_end(35);

    let title_lbl = Label::builder()
        .label(title_text.to_uppercase())
        .css_classes(["cpu-brand"])
        .halign(Align::Center)
        .build();
    
    let title_context = title_lbl.style_context();
    let title_provider = CssProvider::new();
    title_provider.load_from_data("label { font-weight: 900; font-size: 13px; text-transform: uppercase; letter-spacing: 1px; }");
    title_context.add_provider(&title_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    container.append(&title_lbl);
    container
}

fn is_remote_newer(remote: &str, local: &str) -> bool {
    let r_parts: Vec<u32> = remote.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let l_parts: Vec<u32> = local.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    
    for i in 0..std::cmp::max(r_parts.len(), l_parts.len()) {
        let r_val = r_parts.get(i).cloned().unwrap_or(0);
        let l_val = l_parts.get(i).cloned().unwrap_or(0);
        if r_val > l_val { return true; }
        if r_val < l_val { return false; }
    }
    false
}

pub fn build_settings_panel(
    _apply_theme_fn: StdBox<dyn Fn(&str)>, 
    open_dialog_fn: StdBox<dyn Fn(Option<&Window>, &str, &str)>
) -> GtkBox {
    let settings_panel = GtkBox::new(Orientation::Vertical, 0);
    settings_panel.add_css_class("settings-panel-box");
    settings_panel.set_size_request(830, -1);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .height_request(340)
        .build();

    let scroll_content = GtkBox::new(Orientation::Vertical, GAP_LARGE);
    scroll_content.set_margin_bottom(GAP_LARGE);

    let appearance_card = build_card_container("Appearance");
    appearance_card.set_margin_top(GAP_LARGE + 15);

    let theme_flow = FlowBox::new();
    theme_flow.set_valign(Align::Center);
    theme_flow.set_halign(Align::Center);
    theme_flow.set_max_children_per_line(12);
    theme_flow.set_selection_mode(SelectionMode::None);
    theme_flow.set_row_spacing(2u32);
    theme_flow.set_column_spacing(2u32);

    let colors = [
        ("#00e5ff"), ("#00ff99"), ("#ff3366"), ("#ffaa00"),
        ("#b030ff"), ("#ffffff"), ("#e056fd"), ("#ff7675"),
        ("#ff003c"), ("#0984e3"), ("#2ed573"), ("#d63031")
    ];

    let color_buttons_registry: Rc<RefCell<Vec<(String, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    for color in colors {
        let btn = Button::new();
        btn.add_css_class("theme-circle");
        let css = format!("button {{ background-color: {}; }}", color);
        let provider = CssProvider::new();
        provider.load_from_data(&css);
        btn.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let color_clone = color.to_string();
        let registry_weak = Rc::downgrade(&color_buttons_registry);
        btn.connect_clicked(move |_| {
            apply_theme(&color_clone);
            if let Some(registry_rc) = registry_weak.upgrade() {
                for (c_val, b_instance) in registry_rc.borrow().iter() {
                    if c_val == &color_clone {
                        b_instance.set_css_classes(&["theme-circle", "active-theme-circle"]);
                    } else {
                        b_instance.set_css_classes(&["theme-circle"]);
                    }
                }
            }
        });
        theme_flow.insert(&btn, -1);
        color_buttons_registry.borrow_mut().push((color.to_string(), btn));
    }
    appearance_card.append(&theme_flow);

    let app_card = build_card_container("Application");
    let name_lbl = Label::builder().label(APP_NAME).css_classes(["auth-title"]).halign(Align::Center).build();
    let version_lbl = Label::builder().label(&format!("Version: v{}", env!("CARGO_PKG_VERSION"))).css_classes(["auth-desc"]).halign(Align::Center).build();
    let desc_lbl = Label::builder().label(PROJECT_DESCRIPTION).css_classes(["auth-desc"]).halign(Align::Center).wrap(true).build();
    let author_lbl = Label::builder().label(&format!("Author: {}", AUTHOR)).css_classes(["auth-desc"]).halign(Align::Center).build();

    app_card.append(&name_lbl);
    app_card.append(&version_lbl);
    app_card.append(&desc_lbl);
    app_card.append(&author_lbl);

    let links_card = build_card_container("Links & Updates");
    links_card.set_margin_bottom(GAP_LARGE + 25);

    let link_btn = Button::builder().label("Open GitHub").css_classes(["git-btn"]).build();
    link_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("xdg-open").arg(GITHUB_URL).spawn();
    });

    let latest_version_lbl = Label::builder().label("Not Checked").halign(Align::End).build();
    latest_version_lbl.add_css_class("app-about-value");

    let update_btn = Button::builder().label("Check Update").icon_name("software-update-symbolic").focusable(true).build();
    update_btn.add_css_class("boost-btn-style");

    let open_dialog_rc = Rc::new(open_dialog_fn);

    update_btn.connect_clicked(clone!(@weak update_btn, @weak latest_version_lbl => move |_| {
        update_btn.set_sensitive(false);
        update_btn.set_label("Checking...");

        let nested_dialog = open_dialog_rc.clone();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let output = std::process::Command::new("curl")
                .args(&[
                    "-s",
                    "-m", "8",
                    "-H", "User-Agent: PwrAxiom-Client",
                    "https://api.github.com/repos/MR-PR0G/PwrAxiom/releases/latest"
                ])
                .output();
            
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let stdout_str = String::from_utf8_lossy(&out.stdout);
                        if let Ok(release_info) = serde_json::from_str::<GitHubRelease>(&stdout_str) {
                            let _ = tx.send(Ok(release_info.tag_name));
                        } else {
                            let _ = tx.send(Err("ParseError".to_string()));
                        }
                    } else {
                        let _ = tx.send(Err("ServerError".to_string()));
                    }
                }
                Err(_) => {
                    let _ = tx.send(Err("NetworkError".to_string()));
                }
            }
        });

        glib::timeout_add_local(Duration::from_millis(100), move || {
            if let Ok(res) = rx.try_recv() {
                update_btn.set_sensitive(true);
                update_btn.set_label("Check Update");
                let parent_window = update_btn.root().and_then(|r| r.downcast::<Window>().ok());

                match res {
                    Ok(tag_name) => {
                        let remote_tag = tag_name.trim().trim_start_matches('v').to_string();
                        let native_tag = env!("CARGO_PKG_VERSION").trim().trim_start_matches('v');

                        latest_version_lbl.set_label(&tag_name);

                        if remote_tag == native_tag {
                            (*nested_dialog)(parent_window.as_ref(), "System Status", "Your Core system architecture is currently up to date.");
                        } else if is_remote_newer(&remote_tag, native_tag) {
                            (*nested_dialog)(parent_window.as_ref(), "Update Available", &format!("A newer runtime build (v{}) is ready on global repository sync trees.", remote_tag));
                        } else {
                            (*nested_dialog)(parent_window.as_ref(), "System Status", "Your running architecture version is newer than the remote stable release stable layer.");
                        }
                    }
                    Err(e) => {
                        if e == "NetworkError" {
                            (*nested_dialog)(parent_window.as_ref(), "Network Error", "Internet connectivity target lost, timeout exceeded or local DNS breakdown.");
                        } else if e == "ParseError" {
                            (*nested_dialog)(parent_window.as_ref(), "Parse Exception", "Signature mapping crash: JSON layout format payload mismatch.");
                        } else {
                            (*nested_dialog)(parent_window.as_ref(), "Access Denied", "GitHub REST infrastructure target rate limits currently reached or server error.");
                        }
                    }
                }
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
    }));

    let git_row = create_row("GitHub Repository", &link_btn);
    let update_row = create_row("Update Management Link", &update_btn);

    links_card.append(&git_row);
    links_card.append(&update_row);
    
    let label_box = GtkBox::new(Orientation::Horizontal, GAP_SMALL);
    label_box.append(&latest_version_lbl);
    links_card.append(&label_box);

    scroll_content.append(&appearance_card);
    scroll_content.append(&app_card);
    scroll_content.append(&links_card);

    scrolled_window.set_child(Some(&scroll_content));
    settings_panel.append(&scrolled_window);

    let active_color = load_theme_from_disk();
    glib::idle_add_local(move || {
        apply_theme(&active_color);
        glib::ControlFlow::Break
    });

    settings_panel
}