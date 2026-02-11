use std::fs;
use std::path::Path;

fn main() {
    // If dashboard/dist/index.html is missing, create a placeholder
    // so rust-embed doesn't fail during cargo build without Node.js
    let dist_dir = Path::new("dashboard/dist");
    let index_path = dist_dir.join("index.html");

    if !index_path.exists() {
        fs::create_dir_all(dist_dir).expect("Failed to create dashboard/dist/");
        fs::write(
            &index_path,
            "<!DOCTYPE html><html><body><h1>Dashboard not built</h1><p>Run <code>make build</code> to build the dashboard.</p></body></html>",
        )
        .expect("Failed to write placeholder index.html");
    }

    // Tell cargo to re-run if dashboard build output changes
    println!("cargo:rerun-if-changed=dashboard/dist/index.html");
}
