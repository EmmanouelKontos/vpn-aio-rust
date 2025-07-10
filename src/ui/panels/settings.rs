use eframe::egui;
use crate::config::Config;
use crate::system::{SystemInfo, installer::PackageInstaller, updater::{AppUpdater, UpdateInfo}};
use crate::ui::components::{Card, GlassButton};
use crate::ui::theme::Theme;

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, system_info: &mut SystemInfo, package_installer: &PackageInstaller, app_updater: &AppUpdater, update_info: &mut Option<UpdateInfo>) {
        let theme = Theme::new();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Settings");
            ui.add_space(20.0);
        
        Self::draw_appearance_card(ui, &theme, config);
        ui.add_space(16.0);
        
        Self::draw_vpn_settings_card(ui, &theme, config);
        ui.add_space(16.0);
        
        Self::draw_system_info_card(ui, &theme, system_info);
        ui.add_space(16.0);
        
        Self::draw_dependencies_card(ui, &theme, system_info, package_installer);
        ui.add_space(16.0);
        
        Self::draw_updates_card(ui, &theme, app_updater, update_info);
        ui.add_space(16.0);
        
            Self::draw_about_card(ui, &theme);
        });
    }
    
    fn draw_appearance_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config) {
        Card::show(ui, theme, "Appearance", |ui| {
            ui.horizontal(|ui| {
                ui.label("Theme:");
                ui.add_space(12.0);
                
                if ui.selectable_label(config.dark_mode, "Dark").clicked() {
                    config.dark_mode = true;
                }
                
                if ui.selectable_label(!config.dark_mode, "Light").clicked() {
                    config.dark_mode = false;
                }
            });
            
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Restart required for theme changes to take effect").color(theme.text_secondary));
        });
    }
    
    fn draw_vpn_settings_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config) {
        Card::show(ui, theme, "VPN Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Auto-connect to VPN on startup:");
                ui.add_space(12.0);
                
                ui.checkbox(&mut config.auto_connect_vpn, "Enable auto-connect");
            });
            
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Auto-connect will use the first available VPN configuration").color(theme.text_secondary));
        });
    }
    
    fn draw_system_info_card(ui: &mut egui::Ui, theme: &Theme, system_info: &SystemInfo) {
        Card::show(ui, theme, "System Information", |ui| {
            ui.horizontal(|ui| {
                ui.label("Distribution:");
                ui.label(egui::RichText::new(&system_info.distribution).color(theme.text_secondary));
            });
            
            ui.horizontal(|ui| {
                ui.label("Package Manager:");
                let pm_name = match system_info.package_manager {
                    crate::system::PackageManager::Apt => "APT (Debian/Ubuntu)",
                    crate::system::PackageManager::Pacman => "Pacman (Arch Linux)",
                    crate::system::PackageManager::Dnf => "DNF (Fedora)",
                    crate::system::PackageManager::Yum => "YUM (CentOS/RHEL)",
                    crate::system::PackageManager::Zypper => "Zypper (openSUSE)",
                    crate::system::PackageManager::Unknown => "Unknown",
                    crate::system::PackageManager::Chocolatey => "Chocolatey (Windows)",
                    crate::system::PackageManager::Scoop => "Scoop (Windows)",
                    crate::system::PackageManager::Winget => "Winget (Windows)",
                };
                ui.label(egui::RichText::new(pm_name).color(theme.text_secondary));
            });
        });
    }
    
    fn draw_dependencies_card(ui: &mut egui::Ui, theme: &Theme, system_info: &mut SystemInfo, package_installer: &PackageInstaller) {
        Card::show(ui, theme, "Dependencies", |ui| {
            ui.label("System dependencies status:");
            ui.add_space(8.0);
            
            let mut missing_packages = Vec::new();
            
            for dep in &system_info.dependencies {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", dep.name));
                    
                    if dep.is_installed {
                        ui.label(egui::RichText::new("✓ Installed").color(theme.success));
                        if let Some(version) = &dep.version {
                            ui.label(egui::RichText::new(format!("({})", version)).color(theme.text_secondary));
                        }
                    } else {
                        ui.label(egui::RichText::new("✗ Missing").color(theme.error));
                        
                        // Only add to missing packages if it's not a built-in tool
                        if dep.package_name != "builtin" {
                            missing_packages.push(dep.package_name.clone());
                        }
                        
                        if dep.required {
                            ui.label(egui::RichText::new("(Required)").color(theme.error));
                        }
                    }
                });
            }
            
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if GlassButton::show(ui, theme, "Refresh Dependencies", true).clicked() {
                    // Live refresh dependencies
                    if let Err(e) = system_info.refresh_dependencies() {
                        log::error!("Failed to refresh dependencies: {}", e);
                    }
                }
                ui.label(egui::RichText::new("Click refresh after installing new dependencies").color(theme.text_secondary));
            });
            
            if !missing_packages.is_empty() {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                ui.label("Missing packages can be installed with:");
                ui.add_space(4.0);
                
                let install_command = package_installer.get_install_command(&missing_packages);
                ui.code(&install_command);
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    if GlassButton::show(ui, theme, "Copy Install Command", true).clicked() {
                        ui.output_mut(|o| o.copied_text = install_command.clone());
                    }
                    
                    if GlassButton::show(ui, theme, "Open Terminal/PowerShell", true).clicked() {
                        // Open terminal with the command ready to run
                        #[cfg(windows)]
                        {
                            let _ = std::process::Command::new("cmd")
                                .args(&["/c", "start", "cmd"])
                                .spawn();
                        }
                        
                        #[cfg(unix)]
                        {
                            let _ = std::process::Command::new("gnome-terminal")
                                .spawn()
                                .or_else(|_| std::process::Command::new("xterm").spawn())
                                .or_else(|_| std::process::Command::new("konsole").spawn());
                        }
                    }
                });
                
                // Show package manager installation help for Windows
                if install_command.contains("# No package manager found") {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("No package manager detected. Install one first:").strong());
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        if GlassButton::show(ui, theme, "Install Winget", true).clicked() {
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(&["/c", "start", "https://aka.ms/getwinget"])
                                    .spawn();
                            }
                        }
                        
                        if GlassButton::show(ui, theme, "Install Chocolatey", true).clicked() {
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(&["/c", "start", "https://chocolatey.org/install"])
                                    .spawn();
                            }
                        }
                        
                        if GlassButton::show(ui, theme, "Install Scoop", true).clicked() {
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(&["/c", "start", "https://scoop.sh/"])
                                    .spawn();
                            }
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Or download directly from the links shown above").color(theme.text_secondary));
                } else {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Copy the command above and run it in your terminal/PowerShell as administrator").color(theme.text_secondary));
                }
            }
        });
    }
    
    fn draw_updates_card(ui: &mut egui::Ui, theme: &Theme, _app_updater: &AppUpdater, update_info: &mut Option<UpdateInfo>) {
        Card::show(ui, theme, "Updates", |ui| {
            ui.horizontal(|ui| {
                ui.label("Current Version:");
                ui.label(egui::RichText::new(env!("CARGO_PKG_VERSION")).color(theme.text_secondary));
            });
            
            ui.add_space(8.0);
            
            if let Some(update) = update_info {
                if update.update_available {
                    ui.label(egui::RichText::new(format!("New version available: {}", update.latest_version)).color(theme.success));
                    ui.add_space(4.0);
                    
                    if !update.release_notes.is_empty() {
                        ui.label("Release Notes:");
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(&update.release_notes).color(theme.text_secondary));
                            });
                    }
                    
                    ui.add_space(8.0);
                    
                    if GlassButton::show(ui, theme, "Download Update", true).clicked() {
                        // This would trigger the update download
                    }
                } else {
                    ui.label(egui::RichText::new("You are using the latest version").color(theme.success));
                }
            } else {
                if GlassButton::show(ui, theme, "Check for Updates", true).clicked() {
                    // This would trigger the update check
                    // For now, just show placeholder
                }
            }
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("Repository:");
                ui.hyperlink("https://github.com/emmanouil/vpn-aio");
            });
        });
    }
    
    fn draw_about_card(ui: &mut egui::Ui, theme: &Theme) {
        Card::show(ui, theme, "About", |ui| {
            ui.label(egui::RichText::new("VPN Manager").size(18.0).strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Version 0.1.0").color(theme.text_secondary));
            ui.add_space(8.0);
            ui.label("A modern VPN, RDP, and Wake-on-LAN management tool");
            ui.add_space(12.0);
            
            ui.label(egui::RichText::new("Features:").strong());
            ui.label("• VPN connection management (OpenVPN & WireGuard)");
            ui.label("• RDP remote desktop connections");
            ui.label("• Wake-on-LAN for remote devices");
            ui.label("• Network device monitoring");
            ui.label("• Dark mode glassy interface");
            ui.label("• Automatic dependency detection");
            ui.label("• Auto-installation of missing dependencies");
            ui.label("• Automatic updates from GitHub");
            
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Built with Rust and egui").color(theme.text_secondary));
        });
    }
}