use eframe::egui;
use log::{info, error, warn};
use std::panic;

mod config;
mod network;
mod system;
mod ui;

use ui::App;

fn main() -> eframe::Result<()> {
    // Initialize logging with better configuration
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Set up panic handler for better error reporting
    panic::set_hook(Box::new(|panic_info| {
        error!("Application panic: {}", panic_info);
        eprintln!("VPN Manager crashed: {}", panic_info);
        eprintln!("Please report this issue at: https://github.com/emmanouil/vpn-aio/issues");
    }));

    info!("Starting VPN Manager v{}", env!("CARGO_PKG_VERSION"));

    // Check system compatibility
    if let Err(e) = check_system_requirements() {
        error!("System requirements check failed: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(true)
            .with_resizable(true)
            .with_title("VPN Manager"),
        ..Default::default()
    };

    info!("Initializing GUI...");
    let result = eframe::run_native(
        "VPN Manager",
        options,
        Box::new(|cc| {
            match initialize_app(cc) {
                Ok(app) => Ok(app),
                Err(e) => {
                    error!("Failed to initialize app: {}", e);
                    std::process::exit(1);
                }
            }
        }),
    );

    match result {
        Ok(()) => {
            info!("VPN Manager exited successfully");
            Ok(())
        }
        Err(e) => {
            error!("VPN Manager exited with error: {}", e);
            Err(e)
        }
    }
}

fn check_system_requirements() -> Result<(), String> {
    // Check if we're on a supported platform
    if !cfg!(target_os = "linux") {
        return Err("This application currently only supports Linux".to_string());
    }

    // Check if we have a display
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        warn!("No display environment detected. Running in headless mode may not work.");
    }

    Ok(())
}

fn initialize_app(cc: &eframe::CreationContext<'_>) -> Result<Box<dyn eframe::App>, String> {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    info!("Image loaders installed successfully");

    match App::new(cc) {
        Ok(app) => {
            info!("Application initialized successfully");
            Ok(Box::new(app))
        }
        Err(e) => {
            error!("Failed to create app: {}", e);
            std::process::exit(1);
        }
    }
}
