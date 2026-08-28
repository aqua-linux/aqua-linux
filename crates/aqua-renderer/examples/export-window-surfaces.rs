use aqua_renderer::{
    encode_png_rgba, render_files_window_rgba_with_theme, render_properties_window_rgba_with_theme,
    render_settings_window_rgba, render_terminal_window_rgba_with_theme,
};
use aqua_shell::{
    AquaTheme, DesktopPropertiesModel, FilesWindowModel, SettingsWindowModel, TerminalView,
};
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
    for theme in AquaTheme::ALL {
        let suffix = theme.id().to_ascii_lowercase();
        let (files, files_probe) =
            render_files_window_rgba_with_theme(640, 420, &FilesWindowModel::default(), theme);
        write_png(
            &output_dir,
            &format!("files-{suffix}.png"),
            640,
            420,
            &files,
        );

        let mut settings_model = SettingsWindowModel {
            theme,
            ..SettingsWindowModel::default()
        };
        settings_model.selected_category = 0;
        let (settings, settings_probe) = render_settings_window_rgba(640, 420, &settings_model);
        write_png(
            &output_dir,
            &format!("settings-{suffix}.png"),
            640,
            420,
            &settings,
        );

        let (terminal, terminal_probe) =
            render_terminal_window_rgba_with_theme(680, 430, &terminal_view, theme);
        write_png(
            &output_dir,
            &format!("terminal-{suffix}.png"),
            680,
            430,
            &terminal,
        );

        let (properties, properties_probe) =
            render_properties_window_rgba_with_theme(480, 300, &properties_model, theme);
        write_png(
            &output_dir,
            &format!("properties-{suffix}.png"),
            480,
            300,
            &properties,
        );

        println!(
            "theme={} files_checksum={:016x}",
            theme.id(),
            files_probe.checksum
        );
        println!(
            "theme={} settings_checksum={:016x}",
            theme.id(),
            settings_probe.checksum
        );
        println!(
            "theme={} terminal_checksum={:016x}",
            theme.id(),
            terminal_probe.checksum
        );
        println!(
            "theme={} properties_checksum={:016x}",
            theme.id(),
            properties_probe.checksum
        );
    }
    println!("window_surface_dir={}", output_dir.display());
}

fn write_png(output_dir: &Path, name: &str, width: u32, height: u32, rgba: &[u8]) {
    fs::write(output_dir.join(name), encode_png_rgba(width, height, rgba))
        .expect("write window surface PNG");
}
