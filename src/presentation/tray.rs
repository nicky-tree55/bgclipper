use log::{error, info, warn};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

use crate::application::clipboard_service::{ClipboardService, ProcessResult};
use crate::domain::port::{ClipboardPort, ConfigPort};

/// User event type for the event loop.
enum UserEvent {
    MenuEvent(MenuEvent),
}

/// Runs the system tray application.
///
/// Creates a tray icon with a context menu (Quit) and listens for a
/// global hotkey to trigger clipboard transparency processing.
///
/// # Panics
///
/// Panics if the event loop, tray icon, or hotkey manager cannot be created.
pub fn run<C, G>(service: ClipboardService<C, G>, hotkey_str: &str)
where
    C: ClipboardPort + 'static,
    G: ConfigPort + 'static,
{
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Build context menu (Quit only)
    let quit_item = MenuItem::new("Quit", true, None);

    let menu = Menu::new();
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

    // Register global hotkey
    let hotkey_manager = GlobalHotKeyManager::new().expect("failed to create hotkey manager");
    let hotkey: HotKey = hotkey_str
        .parse()
        .expect("failed to parse hotkey accelerator");
    hotkey_manager
        .register(hotkey)
        .expect("failed to register global hotkey");
    info!("global hotkey registered: {hotkey_str}");

    let hotkey_id = hotkey.id();

    // Forward menu events to the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let quit_id = quit_item.id().clone();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id() == &quit_id {
                    info!("quit requested");
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::NewEvents(tao::event::StartCause::Poll) => {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id() == hotkey_id {
                        info!("hotkey triggered, processing clipboard");
                        match service.process_clipboard() {
                            Ok(ProcessResult::Processed) => {
                                info!("clipboard image processed successfully");
                            }
                            Ok(ProcessResult::NoImage) => {
                                info!("no image on clipboard");
                            }
                            Err(e) if e.contains("config parse error") => {
                                warn!("config parse error: {e}");
                                show_alert("bgclipper: Config Error", &e);
                            }
                            Err(e) => error!("{e}"),
                        }
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
