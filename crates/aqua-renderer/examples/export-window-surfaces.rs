use aqua_renderer::{
    encode_png_rgba, render_files_window_rgba, render_properties_window_rgba,
    render_settings_window_rgba, render_terminal_window_rgba,
};
use aqua_shell::{DesktopPropertiesModel, FilesWindowModel, SettingsWindowModel, TerminalView};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let output_dir = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/window-surfaces"));
    fs::create_dir_all(&output_dir).expect("create window surface preview directory");

    let (files, files_probe) = render_files_window_rgba(640, 420, &FilesWindowModel::default());
    write_png(&output_dir, "files.png", 640, 420, &files);

    let (settings, settings_probe) =
        render_settings_window_rgba(640, 420, &SettingsWindowModel::default());
    write_png(&output_dir, "settings.png", 640, 420, &settings);

    let terminal_view = TerminalView {
        lines: vec![
            "Aqua Linux".to_string(),
            "aqua@aqua:~$ uname -s".to_string(),
            "Linux".to_string(),
        ],
        cursor_row: 3,
        cursor_col: 0,
        rows: 18,
        cols: 72,
    };
    let (terminal, terminal_probe) = render_terminal_window_rgba(680, 430, &terminal_view);
    write_png(&output_dir, "terminal.png", 680, 430, &terminal);

    let properties_model = DesktopPropertiesModel {
        icon_id: "files",
        title: "Files Properties".to_string(),
        name: "Files",
        kind: "Folder",
        location: "/home/aqua".to_string(),
        status: "Available",
        item_count: Some(4),
        enumeration_capped: false,
        refresh_generation: 1,
    };
    let (properties, properties_probe) = render_properties_window_rgba(480, 300, &properties_model);
    write_png(&output_dir, "properties.png", 480, 300, &properties);

    println!("files_checksum={:016x}", files_probe.checksum);
    println!("settings_checksum={:016x}", settings_probe.checksum);
    println!("terminal_checksum={:016x}", terminal_probe.checksum);
    println!("properties_checksum={:016x}", properties_probe.checksum);
    println!("window_surface_dir={}", output_dir.display());
}

fn write_png(output_dir: &Path, name: &str, width: u32, height: u32, rgba: &[u8]) {
    fs::write(output_dir.join(name), encode_png_rgba(width, height, rgba))
        .expect("write window surface PNG");
}
