use bgclipper::application::clipboard_service::ClipboardService;
use bgclipper::domain::port::ConfigPort;
use bgclipper::infrastructure::clipboard::ArboardClipboardProvider;
use bgclipper::infrastructure::config::TomlConfigProvider;
use bgclipper::presentation::tray;
use log::info;

fn main() {
    // Initialize logger: debug level in debug builds, warn in release
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .format_timestamp_secs()
        .init();

    info!("bgclipper starting");

    let clipboard = ArboardClipboardProvider::new();
    let config = TomlConfigProvider::new().expect("failed to determine config directory");

    // Create default config file if it doesn't exist
    if let Err(e) = config.ensure_config_exists() {
        log::error!("failed to initialize config: {e}");
    }

    info!("config initialized");

    let service = ClipboardService::new(clipboard, config);

    info!("starting system tray event loop");
    tray::run(service);
}
