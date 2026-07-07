use gtk4::prelude::*;
use gtk4::{Button, EventControllerMotion, CssProvider};
use std::rc::Rc;

pub fn apply(btn: &Button, is_active_fn: Box<dyn Fn() -> bool>) {
    let provider = CssProvider::new();
    btn.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);

    let motion = EventControllerMotion::new();
    let provider_clone = provider.clone();
    let is_active_shared: Rc<dyn Fn() -> bool> = Rc::from(is_active_fn);
    let btn_weak = btn.downgrade();

    let is_active_motion = is_active_shared.clone();
    let btn_weak_motion = btn_weak.clone();
    motion.connect_motion(move |_, x, y| {
        if let Some(button) = btn_weak_motion.upgrade() {
            let active = is_active_motion();
            let w = button.width() as f64;
            let h = button.height() as f64;
            let (w, h) = if w > 0.0 && h > 0.0 { (w, h) } else { (210.0, 130.0) };

            let cx = w / 2.0;
            let cy = h / 2.0;
            
            let dx = ((x - cx) / cx).clamp(-1.0, 1.0);
            let dy = ((y - cy) / cy).clamp(-1.0, 1.0);

            let rot_x = dy * -9.5;
            let rot_y = dx * 9.5;

            let border = if active {
                "1.6px solid var(--primary-color)"
            } else {
                "1px solid var(--secondary-color)"
            };

            let shadow = if active {
                "0 6px 16px var(--secondary-color)"
            } else {
                "0 4px 15px rgba(0, 0, 0, 0.4)"
            };

            let css = format!(
                "button {{ 
                    border: {};
                    box-shadow: {};
                    background-image: radial-gradient(circle 120px at {:.1}px {:.1}px, alpha(var(--primary-color), 0.15) 0%, alpha(var(--primary-color), 0.02) 60%, transparent 100%);
                    transform: perspective(800px) scale(1.05) rotateX({:.2}deg) rotateY({:.2}deg);
                    transition: none;
                }}",
                border, shadow, x, y, rot_x, rot_y
            );
            provider_clone.load_from_data(&css);
        }
    });

    let provider_leave = provider.clone();
    let is_active_leave = is_active_shared.clone();
    motion.connect_leave(move |_| {
        let active = is_active_leave();
        let border = if active {
            "1.6px solid var(--primary-color)"
        } else {
            "1px solid var(--secondary-color)"
        };

        let shadow = if active {
            "0 6px 16px var(--secondary-color)"
        } else {
            "0 4px 15px rgba(0, 0, 0, 0.4)"
        };

        let css = format!(
            "button {{ 
                border: {};
                box-shadow: {};
                background-image: none; 
                transform: perspective(800px) scale(1.0) rotateX(0deg) rotateY(0deg); 
                transition: transform 0.22s cubic-bezier(0.25, 0.46, 0.45, 0.94); 
            }}",
            border, shadow
        );
        provider_leave.load_from_data(&css);
    });

    let provider_sync = provider.clone();
    let is_active_sync = is_active_shared.clone();
    btn.connect_local("state-flags-changed", false, move |_args| {
        let active = is_active_sync();
        let border = if active {
            "1.6px solid var(--primary-color)"
        } else {
            "1px solid var(--secondary-color)"
        };

        let shadow = if active {
            "0 6px 16px var(--secondary-color)"
        } else {
            "0 4px 15px rgba(0, 0, 0, 0.4)"
        };

        let css = format!(
            "button {{ 
                border: {};
                box-shadow: {};
            }}",
            border, shadow
        );
        provider_sync.load_from_data(&css);
        None
    });

    btn.add_controller(motion);
}