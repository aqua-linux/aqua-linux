use aqua_renderer::export_desktop_icons_png;
use aqua_shell::{DesktopIconState, DESKTOP_ICON_LAYER_HEIGHT, DESKTOP_ICON_LAYER_WIDTH};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/desktop-icons-overlay.png"));
    let png = export_desktop_icons_png(
        DESKTOP_ICON_LAYER_WIDTH,
        DESKTOP_ICON_LAYER_HEIGHT,
        &DesktopIconState::default(),
    );
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, png)?;
    println!("desktop_icons_png={}", output.display());
    Ok(())
}
