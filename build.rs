fn main() {
    #[cfg(windows)]
    {
        use std::path::Path;
        
        // Check if the icon file exists
        let icon_path = "assets/vpn-aio.ico";
        if Path::new(icon_path).exists() {
            // Compile Windows resources (icon)
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon_path);
            res.set("FileDescription", "VPN Manager - All-in-One VPN Solution");
            res.set("ProductName", "VPN Manager");
            res.set("CompanyName", "VPN Manager Team");
            res.set("LegalCopyright", "Copyright (c) 2024");
            res.set("FileVersion", env!("CARGO_PKG_VERSION"));
            res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
            res.set("InternalName", "vpn-manager.exe");
            res.set("OriginalFilename", "vpn-manager.exe");
            
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            } else {
                println!("cargo:rerun-if-changed={}", icon_path);
                println!("Windows resources compiled successfully");
            }
        } else {
            eprintln!("Warning: Icon file not found at {}", icon_path);
        }
    }
    
    // Tell cargo to rerun if the icon file changes
    println!("cargo:rerun-if-changed=assets/vpn-aio.ico");
    println!("cargo:rerun-if-changed=assets/vpn-aio.png");
}