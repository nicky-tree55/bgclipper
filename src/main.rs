use bgclipper::application::clipboard_service::ClipboardService;
use bgclipper::infrastructure::clipboard::ArboardClipboardProvider;
use bgclipper::infrastructure::config::TomlConfigProvider;
use bgclipper::presentation::tray;

fn main() {
    let clipboard = ArboardClipboardProvider::new();
    let config = TomlConfigProvider::new().expect("failed to determine config directory");
    let service = ClipboardService::new(clipboard, config);

    tray::run(service);
}
