use aqua_renderer::export_pale_wave_wallpaper_png;
use std::{env, fs, path::PathBuf};

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/wallpaper-pale-waves.png"));
    let png = export_pale_wave_wallpaper_png(1536, 1024);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create wallpaper export directory");
    }
    fs::write(&output, png).expect("write pale wave wallpaper PNG");
    println!("pale_wave_wallpaper={}", output.display());
}
