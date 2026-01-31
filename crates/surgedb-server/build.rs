use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let ui_dir = Path::new("ui");
    let dist_dir = Path::new("dist");

    // 1. If we are in a development environment with npm, try to build the UI
    if ui_dir.exists() && ui_dir.join("package.json").exists() {
        println!("cargo:rerun-if-changed=ui/src");
        println!("cargo:rerun-if-changed=ui/index.html");
        println!("cargo:rerun-if-changed=ui/package.json");
        println!("cargo:rerun-if-changed=ui/vite.config.ts");
        println!("cargo:rerun-if-changed=ui/tailwind.config.js");

        // Try to build, but don't fail the whole build if npm is missing
        // This allows people without node to still build the rust parts
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "npm install && npm run build"])
                .current_dir(ui_dir)
                .status()
        } else {
            Command::new("sh")
                .args(["-c", "npm install && npm run build"])
                .current_dir(ui_dir)
                .status()
        };

        if let Ok(s) = status {
            if s.success() {
                println!("cargo:info=UI built successfully");
            } else {
                println!("cargo:warning=UI build failed");
            }
        } else {
            println!("cargo:warning=npm not found, skipping UI build");
        }
    }

    // 2. Ensure the dist directory exists so rust-embed doesn't crash the build
    if !dist_dir.exists() {
        fs::create_dir_all(dist_dir).unwrap();
    }

    // 3. Ensure there is at least an index.html so Assets::get("index.html") works
    let index_path = dist_dir.join("index.html");
    if !index_path.exists() {
        fs::write(index_path, "<html><body><h1>SurgeDB UI not built</h1><p>Please run 'npm install && npm run build' in the ui directory.</p></body></html>").unwrap();
    }
}
