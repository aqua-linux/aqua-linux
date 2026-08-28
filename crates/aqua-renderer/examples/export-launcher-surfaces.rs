use aqua_renderer::{
    export_runtime_desktop_rgba_with_launcher_and_theme, render_pale_wave_wallpaper_rgba,
    ClientLayerPaintPlan,
};
use aqua_scene::Viewport;
use aqua_shell::{AquaTheme, LauncherState};
use std::{env, fs, path::PathBuf};

fn main() {
    let output_dir = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build"));
    fs::create_dir_all(&output_dir).expect("create launcher preview directory");

    let viewport = Viewport::new(1536, 1024);
    let wallpaper = render_pale_wave_wallpaper_rgba(viewport.width, viewport.height);
    let client_plan = ClientLayerPaintPlan {
        status: "client-layer-paint-ready",
        backend: "aqua-software-raster",
        renderer_started: false,
        steps: Vec::new(),
    };

    for theme in AquaTheme::ALL {
        let suffix = theme.id().to_ascii_lowercase();
        let mut launcher = LauncherState::default();
        launcher.open_applications();
        export(
            &output_dir.join(format!("desktop-applications-{suffix}.png")),
            viewport,
            &wallpaper,
            &client_plan,
            &launcher,
            theme,
        );

        launcher.open_search();
        launcher.set_query("settings");
        export(
            &output_dir.join(format!("desktop-search-{suffix}.png")),
            viewport,
            &wallpaper,
            &client_plan,
            &launcher,
            theme,
        );
    }
}

fn export(
    path: &PathBuf,
    viewport: Viewport,
    wallpaper: &[u8],
    client_plan: &ClientLayerPaintPlan,
    launcher: &LauncherState,
    theme: AquaTheme,
) {
    let (frame, probe) = export_runtime_desktop_rgba_with_launcher_and_theme(
        viewport,
        viewport.width,
        viewport.height,
        wallpaper,
        client_plan,
        launcher,
        theme,
    )
    .expect("render launcher preview");
    assert!(probe.is_ready());
    fs::write(path, frame.to_png()).expect("write launcher preview PNG");
    println!(
        "{} mode={} theme={}",
        path.display(),
        probe.mode,
        theme.id()
    );
}
