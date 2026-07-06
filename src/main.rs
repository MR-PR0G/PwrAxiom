use gtk4::prelude::*;
use gtk4::{Application, CssProvider, gdk};
use std::process::ExitCode;
use std::rc::Rc;
use std::cell::RefCell;

pub mod ui; 
pub mod backend;

fn main() -> ExitCode {
    if std::env::var("GSK_RENDERER").is_err() {
        std::env::set_var("GSK_RENDERER", "gl");
    }

    println!("Probing hardware sensors (Bypassing DRM locks)...");
    let receiver = Rc::new(RefCell::new(Some(ui::dashboard::init_and_start_hardware_polling())));
    println!("Hardware probe successful! Starting GUI...");

    let app = Application::builder()
        .application_id("com.pwraxiom.monitor")
        .build();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("ui/style.css"));
        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        ui::dashboard::load_saved_theme();
    });

    app.connect_activate(move |app| {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Power Axiom")
            .default_width(1050)
            .default_height(750)
            .build();

        let overlay = gtk4::Overlay::new();
        
        let rx = receiver.borrow_mut().take().expect("App activated twice?");
        
        let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
        root_box.set_halign(gtk4::Align::Center);
        root_box.set_valign(gtk4::Align::Start);
        root_box.set_margin_top(40);

        let dashboard_widget = ui::dashboard::create(rx);
        root_box.append(&dashboard_widget);

        let controls_widget = ui::controls::create();
        root_box.append(&controls_widget);

        overlay.add_overlay(&root_box);
        window.set_child(Some(&overlay));
        window.present();
    });

    app.run();
    ExitCode::SUCCESS
}