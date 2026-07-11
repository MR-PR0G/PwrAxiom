use gtk4::prelude::*;
use gtk4::{Button, Box, Orientation, Align, Window, Label, PasswordEntry, ProgressBar};
use glib::clone;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::path::PathBuf;
use crate::backend::{BackendManager, Profile};
use crate::ui::spotlight;

thread_local! {
    static ACTIVE_BUTTON: RefCell<Option<Button>> = RefCell::new(None);
}

fn get_profile_config_path() -> PathBuf {
    let mut path = PathBuf::from("/run/user");
    let uid = std::process::id();
    path.push(uid.to_string());
    if !path.exists() {
        path = std::env::temp_dir();
    }
    path.push(".pwraxiom_profile");
    path
}

fn save_profile_to_disk(profile: &str) {
    let _ = std::fs::write(get_profile_config_path(), profile);
}

fn load_profile_from_disk() -> Profile {
    if let Ok(content) = std::fs::read_to_string(get_profile_config_path()) {
        match content.trim() {
            "Custom" => Profile::Custom,
            "Performance" => Profile::Performance,
            "UltraSave" => Profile::UltraSave,
            "Save" => Profile::Save,
            _ => Profile::Balanced,
        }
    } else {
        Profile::Balanced
    }
}

fn open_error_dialog(parent_win: Option<&Window>, title: &str, details: &str) {
    let dialog = Window::builder().title("Execution Error").modal(true).default_width(400).default_height(280).resizable(false).build();
    dialog.add_css_class("auth-win");
    if let Some(parent) = parent_win { dialog.set_transient_for(Some(parent)); }

    let vbox = Box::new(Orientation::Vertical, 12);
    vbox.set_margin_top(25); vbox.set_margin_bottom(25); vbox.set_margin_start(25); vbox.set_margin_end(25);
    vbox.set_valign(Align::Center); vbox.set_halign(Align::Center);

    let err_title = Label::builder().label(title).css_classes(["auth-title"]).build();
    let err_desc = Label::builder().label(details).css_classes(["auth-desc"]).wrap(true).justify(gtk4::Justification::Center).build();
    let close_btn = Button::with_label("Dismiss");
    close_btn.add_css_class("boost-btn-style");
    close_btn.connect_clicked(clone!(@weak dialog => move |_| dialog.close()));

    vbox.append(&err_title); vbox.append(&err_desc); vbox.append(&close_btn);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

fn open_generic_info_dialog(parent_win: Option<&Window>, title: &str, text: &str) {
    let dialog = Window::builder().title(title).modal(true).default_width(360).default_height(180).resizable(false).build();
    dialog.add_css_class("auth-win");
    if let Some(parent) = parent_win { dialog.set_transient_for(Some(parent)); }

    let vbox = Box::new(Orientation::Vertical, 15);
    vbox.set_margin_top(25); vbox.set_margin_bottom(25); vbox.set_margin_start(25); vbox.set_margin_end(25);
    vbox.set_valign(Align::Center); vbox.set_halign(Align::Center);

    let lbl_title = Label::builder().label(title).css_classes(["auth-title"]).build();
    let lbl_desc = Label::builder().label(text).css_classes(["auth-desc"]).wrap(true).justify(gtk4::Justification::Center).build();
    
    let close_btn = Button::with_label("Confirm");
    close_btn.add_css_class("boost-btn-style");
    close_btn.connect_clicked(clone!(@weak dialog => move |_| dialog.close()));

    vbox.append(&lbl_title); vbox.append(&lbl_desc); vbox.append(&close_btn);
    dialog.set_child(Some(&vbox));
    dialog.present();
}

fn mark_button_active(btn: &Button) {
    ACTIVE_BUTTON.with(|cell| {
        if let Some(old_btn) = cell.borrow_mut().take() {
            old_btn.remove_css_class("active-planet");
            old_btn.queue_allocate();
        }
        btn.add_css_class("active-planet");
        btn.queue_allocate();
        *cell.borrow_mut() = Some(btn.clone());
    });
}

fn setup_button_spotlight(btn: &Button) {
    let btn_weak = btn.downgrade();
    spotlight::apply(btn, std::boxed::Box::new(move || {
        if let Some(b) = btn_weak.upgrade() {
            return ACTIVE_BUTTON.with(|cell| cell.borrow().as_ref() == Some(&b));
        }
        false
    }));
}

fn dispatch_profile_application(target_btn: &Button, target_enum: Profile, status_lbl: &Label, progress_bar: &ProgressBar, sub_status_lbl: &Label, dialog: &Window, root_win_weak: Option<glib::WeakRef<Window>>) {
    let step = Rc::new(RefCell::new(0));
    let (tx, rx) = std::sync::mpsc::channel();
    let rx_ref = Rc::new(RefCell::new(Some(rx)));

    glib::timeout_add_local(Duration::from_millis(600), clone!(@weak dialog, @weak status_lbl, @weak progress_bar, @weak sub_status_lbl, @strong target_btn, @strong root_win_weak => @default-return glib::ControlFlow::Break, move || {
        let mut curr = step.borrow_mut();
        *curr += 1;
        match *curr {
            1 => {
                match target_enum {
                    Profile::UltraSave => sub_status_lbl.set_label("Calculating 25% low-power core limits..."),
                    Profile::Save => sub_status_lbl.set_label("Calculating 75% active workload paths..."),
                    Profile::Performance => sub_status_lbl.set_label("Overclocking margins to maximum ceilings..."),
                    _ => sub_status_lbl.set_label("Wiping old CPU scaling boundaries..."),
                }
                progress_bar.set_fraction(0.4);
                glib::ControlFlow::Continue
            }
            2 => {
                match target_enum {
                    Profile::UltraSave => sub_status_lbl.set_label("Clamps clock frequencies to 1.5 GHz..."),
                    Profile::Save => sub_status_lbl.set_label("Capping operational frequencies to 50%..."),
                    Profile::Performance => sub_status_lbl.set_label("Enabling dynamic CPU Turbo Boost states..."),
                    _ => sub_status_lbl.set_label("Injecting native scheduler configurations..."),
                }
                progress_bar.set_fraction(0.7);

                let tx_clone = tx.clone();
                std::thread::spawn(move || {
                    if let Ok(mut manager) = BackendManager::global().lock() {
                        let res = manager.apply_profile(target_enum);
                        manager.set_password(String::new());
                        let _ = tx_clone.send(res);
                    } else {
                        let _ = tx_clone.send(Err("Failed to acquire thread lock architecture".to_string()));
                    }
                });

                glib::ControlFlow::Continue
            }
            3 => {
                let channel_rx = rx_ref.borrow_mut().take();
                if let Some(rx_channel) = channel_rx {
                    match rx_channel.recv_timeout(Duration::from_secs(10)) {
                        Ok(Ok(())) => {
                            let profile_str = match target_enum {
                                Profile::Custom => "Custom",
                                Profile::Performance => "Performance",
                                Profile::UltraSave => "UltraSave",
                                Profile::Save => "Save",
                                Profile::Balanced => "Balanced",
                            };
                            save_profile_to_disk(profile_str);

                            status_lbl.set_label("Optimization Completed");
                            sub_status_lbl.set_label("Core parameters successfully deployed.");
                            progress_bar.set_fraction(1.0);
                            mark_button_active(&target_btn);
                        }
                        _ => {
                            dialog.close();
                            let parent_win = root_win_weak.as_ref().and_then(|w| w.upgrade());
                            open_error_dialog(parent_win.as_ref(), "Privilege Error", "Incorrect root password or sudo access denied. Check terminal logs.");
                            return glib::ControlFlow::Break;
                        }
                    }
                }
                glib::ControlFlow::Continue
            }
            _ => {
                dialog.close();
                glib::ControlFlow::Break
            }
        }
    }));
}

fn open_auth_dialog(parent_box: &Box, target_btn: &Button, profile_name: &str, description: &str, target_enum: Profile) {
    let root_win_opt = parent_box.root().and_then(|r| r.downcast::<Window>().ok());
    let root_win_weak = root_win_opt.as_ref().map(|w| w.downgrade());

    let is_already_active = ACTIVE_BUTTON.with(|cell| cell.borrow().as_ref() == Some(target_btn));
    if is_already_active {
        open_generic_info_dialog(root_win_opt.as_ref(), "Profile Active", "This mode is currently selected. If you want to reset all limits, please select the 'Balanced' profile.");
        return;
    }

    let dialog = Window::builder().title("Authentication Required").modal(true).default_width(420).default_height(280).resizable(false).build();
    dialog.add_css_class("auth-win");
    if let Some(ref root_win) = root_win_opt { dialog.set_transient_for(Some(root_win)); }

    let main_container = Box::new(Orientation::Vertical, 16);
    main_container.set_margin_top(25); main_container.set_margin_bottom(25); main_container.set_margin_start(25); main_container.set_margin_end(25);
    main_container.set_valign(Align::Center); main_container.set_halign(Align::Center);

    let title_lbl = Label::builder().label(&format!("Apply Profile: {}", profile_name)).css_classes(["auth-title"]).halign(Align::Center).build();
    let desc_lbl = Label::builder().label(description).css_classes(["auth-desc"]).halign(Align::Center).wrap(true).justify(gtk4::Justification::Center).max_width_chars(38).build();

    main_container.append(&title_lbl); main_container.append(&desc_lbl);
    let interaction_box = Box::new(Orientation::Vertical, 12);
    let password_entry = PasswordEntry::builder().placeholder_text("Enter Administrator Password").css_classes(["auth-entry"])
    .width_request(300).activates_default(true).build();
    password_entry.set_alignment(0.5);

    let btn_box = Box::new(Orientation::Horizontal, 12);
    btn_box.set_halign(Align::Center);
    let cancel_btn = Button::with_label("Cancel"); cancel_btn.add_css_class("git-btn");
    let auth_btn = Button::with_label("Authenticate"); auth_btn.add_css_class("boost-btn-style");

    cancel_btn.connect_clicked(clone!(@weak dialog => move |_| dialog.close()));
    
    auth_btn.connect_clicked(clone!(@weak dialog, @weak password_entry, @weak main_container, @weak interaction_box, @strong target_btn, @strong root_win_weak => move |_| {
        let password = password_entry.text().to_string();
        let password_trim = password.trim();
        if password_trim.is_empty() { return; }
        
        if let Ok(mut manager) = BackendManager::global().lock() { manager.set_password(password_trim.to_string()); }
        main_container.remove(&interaction_box);
        
        let status_lbl = Label::builder().label("Applying core rules optimization...").css_classes(["auth-title"]).build();
        let progress_bar = ProgressBar::builder().css_classes(["mem-bar"]).hexpand(true).width_request(320).build();
        progress_bar.set_fraction(0.1);
        let sub_status_lbl = Label::builder().label("Probing target topologies").css_classes(["auth-sub"]).build();
        
        let status_box = Box::new(Orientation::Vertical, 10);
        status_box.set_width_request(340);
        status_box.set_halign(Align::Center);
        status_box.append(&status_lbl); status_box.append(&progress_bar); status_box.append(&sub_status_lbl);
        main_container.append(&status_box);

        dispatch_profile_application(&target_btn, target_enum, &status_lbl, &progress_bar, &sub_status_lbl, &dialog, root_win_weak.clone());
    }));

    password_entry.connect_activate(clone!(@weak auth_btn => move |_| { auth_btn.activate(); }));
    interaction_box.append(&password_entry); btn_box.append(&cancel_btn); btn_box.append(&auth_btn);
    interaction_box.append(&btn_box); main_container.append(&interaction_box);
    dialog.set_child(Some(&main_container));
    dialog.present();
}

pub fn create() -> Box {
    let main_vbox = Box::new(Orientation::Vertical, 0);
    let top_box = Box::new(Orientation::Horizontal, 20);
    top_box.set_halign(Align::Center); top_box.set_valign(Align::Start);

    let btn_custom_setup = Button::builder().label("Custom\nSetup").css_classes(["planet", "planet-left"]).margin_top(45).focusable(false).build();
    let btn_balanced = Button::builder().label("Balanced").css_classes(["planet"]).margin_top(0).focusable(false).build();
    let btn_perf = Button::builder().label("Performance").css_classes(["planet", "planet-right"]).margin_top(45).focusable(false).build();

    let top_buttons = [&btn_custom_setup, &btn_balanced, &btn_perf];
    for btn in top_buttons.iter() {
        if let Some(child) = btn.child() {
            if let Ok(label) = child.downcast::<gtk4::Label>() { label.set_justify(gtk4::Justification::Center); }
        }
        setup_button_spotlight(btn);
    }

    top_box.append(&btn_custom_setup); top_box.append(&btn_balanced); top_box.append(&btn_perf);

    let bottom_box = Box::new(Orientation::Horizontal, 50);
    bottom_box.set_halign(Align::Center); bottom_box.set_margin_top(25);

    let btn_ultra_save = Button::builder().label("Ultra\nSave").css_classes(["planet"]).focusable(false).build();
    let btn_save = Button::builder().label("Save").css_classes(["planet"]).focusable(false).build();

    let bottom_buttons = [&btn_ultra_save, &btn_save];
    for btn in bottom_buttons.iter() {
        if let Some(child) = btn.child() {
            if let Ok(label) = child.downcast::<gtk4::Label>() { label.set_justify(gtk4::Justification::Center); }
        }
        setup_button_spotlight(btn);
    }

    bottom_box.append(&btn_ultra_save); bottom_box.append(&btn_save);

    let saved_profile = load_profile_from_disk();
    match saved_profile {
        Profile::Custom => mark_button_active(&btn_custom_setup),
        Profile::Balanced => mark_button_active(&btn_balanced),
        Profile::Performance => mark_button_active(&btn_perf),
        Profile::UltraSave => mark_button_active(&btn_ultra_save),
        Profile::Save => mark_button_active(&btn_save),
    }

    btn_custom_setup.connect_clicked(|btn| {
        if let Some(root_w) = btn.root().and_then(|r| r.downcast::<Window>().ok()) {
            open_generic_info_dialog(Some(&root_w), "Custom Setup", "This feature will be enabled in future versions.");
        }
    });
    
    btn_perf.connect_clicked(clone!(@weak main_vbox, @strong btn_perf => move |_| {
        open_auth_dialog(&main_vbox, &btn_perf, "Performance", "Forces processors into continuous peak frequency bounds and activates maximum system execution bounds.", Profile::Performance);
    }));
    
    btn_save.connect_clicked(clone!(@weak main_vbox, @strong btn_save => move |_| {
        open_auth_dialog(&main_vbox, &btn_save, "Save", "Adjusts maximum computing frequencies and disables high energy operational lines dynamically.", Profile::Save);
    }));

    btn_balanced.connect_clicked(clone!(@weak main_vbox, @strong btn_balanced => move |_| {
        open_auth_dialog(&main_vbox, &btn_balanced, "Balanced", "Restores CPU governor profiles to default behaviors and balances current dynamic operational boundaries.", Profile::Balanced);
    }));

    btn_ultra_save.connect_clicked(clone!(@weak main_vbox, @strong btn_ultra_save => move |_| {
        open_auth_dialog(&main_vbox, &btn_ultra_save, "Ultra Save", "Clamps general processing frequency ceilings and sets core execution metrics to extreme preservation lines.", Profile::UltraSave);
    }));

    main_vbox.append(&top_box); main_vbox.append(&bottom_box);
    main_vbox
}