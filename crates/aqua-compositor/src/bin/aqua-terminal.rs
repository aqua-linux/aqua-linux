#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;

    if std::env::args().nth(1).as_deref() == Some("--probe-pty") {
        if let Err(error) = aqua_compositor::probe_aqua_terminal_pty() {
            eprintln!("aqua-terminal: {error}");
            std::process::exit(1);
        }
        return;
    }

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

    if let Err(error) = aqua_compositor::run_aqua_terminal_client(socket_path) {
        eprintln!("aqua-terminal: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("aqua-terminal requires Linux and the Aqua Wayland session");
    std::process::exit(1);
}
