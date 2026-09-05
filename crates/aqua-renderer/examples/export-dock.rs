use aqua_renderer::export_dock_png;
use aqua_shell::DockState;
use std::{env, fs, path::PathBuf};

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/dock-overlay.png"));
    let state = DockState {
        applications_open: false,
        search_open: false,
        files_running: true,
        settings_running: true,
        active_workspace: 0,
    };
    let png = export_dock_png(1232, 64, &state);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create dock export directory");
    }
    fs::write(&output, png).expect("write dock overlay PNG");
    println!("dock_overlay={}", output.display());
}
