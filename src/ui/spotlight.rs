use gtk4::prelude::*;
use gtk4::{Button, EventControllerMotion, CssProvider, GestureClick};

pub fn apply(btn: &Button, is_active_fn: Box<dyn Fn() -> bool>) {
    let provider = CssProvider::new();
    btn.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);

    let motion = EventControllerMotion::new();
    let provider_clone = provider.clone();

    motion.connect_motion(move |controller, x, y| {
        let active = is_active_fn();
        let widget = controller.widget();
        let w = widget.width() as f64;
        let h = widget.height() as f64;
        let (w, h) = if w > 0.0 && h > 0.0 { (w, h) } else { (210.0, 130.0) };

        let cx = w / 2.0;
        let cy = h / 2.0;
        let dx = (x - cx) / cx;
        let dy = (y - cy) / cy;

        let rot_x = dy * -9.5;
        let rot_y = dx * 9.5;

        let css = if active {
            format!(
                "button {{ 
                    background-image: radial-gradient(circle 125px at {:.1}px {:.1}px, rgba(255, 255, 255, 0.25) 0%, rgba(255, 255, 255, 0.04) 55%, transparent 100%);
                    transform: perspective(800px) scale(1.05) rotateX({:.2}deg) rotateY({:.2}deg);
                    transition: none;
                }}",
                x, y, rot_x, rot_y
            )
        } else {
            format!(
                "button {{ 
                    background-image: radial-gradient(circle 115px at {:.1}px {:.1}px, rgba(255, 255, 255, 0.16) 0%, rgba(255, 255, 255, 0.02) 50%, transparent 100%);
                    transform: perspective(800px) scale(1.05) rotateX({:.2}deg) rotateY({:.2}deg);
                    transition: none;
                }}",
                x, y, rot_x, rot_y
            )
        };
        provider_clone.load_from_data(&css);
    });

    let provider_leave = provider.clone();
    motion.connect_leave(move |_| {
        provider_leave.load_from_data(
            "button { 
                background-image: none; 
                transform: perspective(800px) scale(1.0) rotateX(0deg) rotateY(0deg); 
                transition: transform 0.22s cubic-bezier(0.25, 0.46, 0.45, 0.94); 
            }"
        );
    });
    btn.add_controller(motion);

    let click_gesture = GestureClick::new();
    let provider_press = provider.clone();
    
    click_gesture.connect_pressed(move |_, _, _, _| {
        provider_press.load_from_data(
            "button { 
                transform: perspective(800px) scale(0.94) translateY(6px); 
                box-shadow: inset 0 8px 25px rgba(0,0,0,0.85); 
                transition: transform 0.1s ease-in-out; 
            }"
        );
    });
    
    let provider_release = provider.clone();
    click_gesture.connect_released(move |_, _, _, _| {
        provider_release.load_from_data(
            "button { 
                transform: perspective(800px) scale(1.05); 
                box-shadow: 0 16px 40px rgba(0,0,0,0.45); 
                transition: transform 0.2s cubic-bezier(0.25, 0.8, 0.25, 1); 
            }"
        );
    });
    btn.add_controller(click_gesture);
}