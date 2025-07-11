#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    // In release mode on Windows, log to file instead of console
    #[cfg(all(windows, not(debug_assertions)))]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        // Create logs directory
        let log_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("vpn-manager");
        std::fs::create_dir_all(&log_dir).ok();
        
        // Set up file logging
        let log_file = log_dir.join("vpn-manager.log");
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .target(env_logger::Target::Pipe(Box::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .unwrap()
            )))
            .init();
    }
    
    #[cfg(not(all(windows, not(debug_assertions))))]
    {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
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

    let icon_data = include_bytes!("../assets/vpn-aio.png");
    let icon = eframe::icon_data::from_png_bytes(icon_data).unwrap_or_default();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 650.0])
            .with_min_inner_size([700.0, 500.0])
            .with_decorations(true)
            .with_resizable(true)
            .with_title("VPN Manager")
            .with_icon(icon),
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
    if !cfg!(any(target_os = "linux", target_os = "windows", target_os = "macos")) {
        return Err("This application currently supports Linux, Windows, and macOS".to_string());
    }

    // Check if we have a display (Linux/Unix specific)
    #[cfg(unix)]
    {
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            warn!("No display environment detected. Running in headless mode may not work.");
        }
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
