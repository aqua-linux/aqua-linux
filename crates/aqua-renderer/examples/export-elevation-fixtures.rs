use aqua_renderer::{
    elevation_acceptance_report, encode_png_rgba, render_elevation_acceptance_rgba,
};
use aqua_scene::Viewport;
use aqua_shell::AquaTheme;
use aqua_text::OutputScale;
use std::{env, fs, path::PathBuf};

fn main() {
    if let Some(output_dir) = env::args_os().nth(1).map(PathBuf::from) {
        fs::create_dir_all(&output_dir).expect("create elevation fixture directory");
        let viewport = Viewport::new(1280, 800);
        for theme in AquaTheme::ALL {
            for scale in OutputScale::ALL {
                let (rgba, probe) =
                    render_elevation_acceptance_rgba(viewport, theme, scale).unwrap();
                assert!(probe.is_ready());
                let filename = format!(
                    "elevation-{}-{}-{}.png",
                    theme.id().to_ascii_lowercase(),
                    scale.numerator(),
                    scale.denominator(),
                );
                fs::write(
                    output_dir.join(filename),
                    encode_png_rgba(viewport.width, viewport.height, &rgba),
                )
                .expect("write elevation fixture PNG");
            }
        }
    }
    print!("{}", elevation_acceptance_report());
}
