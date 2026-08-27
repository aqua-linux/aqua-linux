#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;

    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/run/aqua"));
    let display = std::env::var_os("WAYLAND_DISPLAY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("aqua-wayland-drm-0"));
    let socket_path = if display.is_absolute() {
        display
    } else {
        runtime_dir.join(display)
    };

    if let Err(error) = aqua_compositor::run_aqua_files_client(socket_path) {
        eprintln!("aqua-files: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("aqua-files requires Linux and the Aqua Wayland session");
    std::process::exit(1);
}
