use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};

use crate::application::clipboard_service::{ClipboardService, ProcessResult};
use crate::domain::port::{ClipboardPort, ConfigPort};

/// Clipboard polling interval when enabled.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// User event type for the event loop.
enum UserEvent {
    MenuEvent(MenuEvent),
}

/// Runs the system tray application.
///
/// Creates a tray icon with a context menu (Enable/Disable, Quit) and
/// polls the clipboard at regular intervals when enabled.
///
/// # Panics
///
/// Panics if the event loop or tray icon cannot be created.
pub fn run<C, G>(service: ClipboardService<C, G>)
where
    C: ClipboardPort + 'static,
    G: ConfigPort + 'static,
{
    let enabled = Arc::new(AtomicBool::new(true));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Build context menu
    let toggle_item = MenuItem::new("Disable", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let menu = Menu::new();
    menu.append(&toggle_item).expect("failed to add menu item");
    menu.append(&quit_item).expect("failed to add menu item");

    // Build tray icon (no custom icon — uses default)
    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("bgclipper")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to create tray icon");

    // Forward menu events to the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + POLL_INTERVAL);

        match event {
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id() == &toggle_id {
                    let was_enabled = enabled.fetch_xor(true, Ordering::Relaxed);
                    if was_enabled {
                        toggle_item.set_text("Enable");
                    } else {
                        toggle_item.set_text("Disable");
                    }
                } else if event.id() == &quit_id {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::NewEvents(tao::event::StartCause::ResumeTimeReached { .. }) => {
                if enabled.load(Ordering::Relaxed) {
                    match service.process_clipboard() {
                        Ok(ProcessResult::Processed) => {}
                        Ok(ProcessResult::NoImage) => {}
                        Err(e) => eprintln!("bgclipper: {e}"),
                    }
                }
            }
            _ => {}
        }
    });
}
