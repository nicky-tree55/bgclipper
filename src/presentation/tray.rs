use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::{error, info, warn};
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

    // Load tray icon from embedded PNG
    let icon_bytes = include_bytes!("../../logo/tray_icon.png");
    let icon_image = image::load_from_memory(icon_bytes).expect("failed to load tray icon");
    let icon_rgba = icon_image.to_rgba8();
    let (icon_w, icon_h) = icon_rgba.dimensions();
    let icon =
        tray_icon::Icon::from_rgba(icon_rgba.into_raw(), icon_w, icon_h).expect("invalid icon");

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("bgclipper")
        .with_icon(icon)
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
                        info!("monitoring disabled");
                    } else {
                        toggle_item.set_text("Disable");
                        info!("monitoring enabled");
                    }
                } else if event.id() == &quit_id {
                    info!("quit requested");
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::NewEvents(tao::event::StartCause::ResumeTimeReached { .. }) => {
                if enabled.load(Ordering::Relaxed) {
                    match service.process_clipboard() {
                        Ok(ProcessResult::Processed) => {
                            info!("clipboard image processed successfully");
                        }
                        Ok(ProcessResult::NoImage) => {}
                        Err(e) if e.contains("config parse error") => {
                            warn!("config parse error: {e}");
                            show_alert("bgclipper: Config Error", &e);
                            // Disable processing until user fixes config
                            enabled.store(false, Ordering::Relaxed);
                            toggle_item.set_text("Enable");
                        }
                        Err(e) => error!("{e}"),
                    }
                }
            }
            _ => {}
        }
    });
}

/// Shows a native alert dialog.
///
/// Uses `osascript` on macOS and `msg` on Windows as a simple cross-platform approach.
fn show_alert(title: &str, message: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"display dialog "{}" with title "{}" buttons {{"OK"}} default button "OK""#,
            message.replace('"', r#"\""#),
            title.replace('"', r#"\""#),
        );
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("msg")
            .args(["*", &format!("{title}\n\n{message}")])
            .spawn();
    }

    error!("{title}: {message}");
}
