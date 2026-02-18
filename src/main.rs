use bgclipper::application::clipboard_service::ClipboardService;
use bgclipper::domain::color::Color;
use bgclipper::domain::port::ConfigPort;
use bgclipper::infrastructure::clipboard::ArboardClipboardProvider;
use bgclipper::infrastructure::config::TomlConfigProvider;
use bgclipper::presentation::tray;

fn main() {
    let clipboard = ArboardClipboardProvider::new();
    let config = TomlConfigProvider::new().expect("failed to determine config directory");

    // Ensure config file exists with default settings
    if config.load_target_color().is_ok()
        && let Err(e) = config.save_target_color(&Color::default())
    {
        eprintln!("bgclipper: failed to initialize config: {e}");
    }

    let service = ClipboardService::new(clipboard, config);

    tray::run(service);
}
