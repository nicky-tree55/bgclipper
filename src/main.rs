use bgclipper::application::clipboard_service::ClipboardService;
use bgclipper::domain::port::ConfigPort;
use bgclipper::infrastructure::clipboard::ArboardClipboardProvider;
use bgclipper::infrastructure::config::TomlConfigProvider;
use bgclipper::presentation::tray;

fn main() {
    let clipboard = ArboardClipboardProvider::new();
    let config = TomlConfigProvider::new().expect("failed to determine config directory");

    // Create default config file if it doesn't exist
    if let Err(e) = config.ensure_config_exists() {
        eprintln!("bgclipper: failed to initialize config: {e}");
    }

    let service = ClipboardService::new(clipboard, config);

    tray::run(service);
}
