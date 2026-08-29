use aqua_renderer::{
    encode_png_rgba, render_typography_layout_acceptance_rgba, typography_layout_acceptance_report,
};
use aqua_scene::Viewport;
use aqua_shell::AquaTheme;
use aqua_text::OutputScale;
use std::{env, fs, path::PathBuf};

fn main() {
    if let Some(output_dir) = env::args_os().nth(1).map(PathBuf::from) {
        fs::create_dir_all(&output_dir).expect("create typography fixture directory");
        for (viewport, scale) in [
            (Viewport::new(800, 600), OutputScale::One),
            (Viewport::new(1280, 800), OutputScale::One),
            (Viewport::new(1536, 1024), OutputScale::FiveQuarters),
        ] {
            for theme in AquaTheme::ALL {
                let (rgba, probe) =
                    render_typography_layout_acceptance_rgba(viewport, theme, scale)
                        .expect("supported typography fixture viewport");
                assert!(probe.is_ready());
                let filename = format!(
                    "typography-{}x{}-{}.png",
                    viewport.width,
                    viewport.height,
                    theme.id().to_ascii_lowercase()
                );
                fs::write(
                    output_dir.join(filename),
                    encode_png_rgba(viewport.width, viewport.height, &rgba),
                )
                .expect("write typography fixture PNG");
            }
        }
    }
    print!("{}", typography_layout_acceptance_report());
}
