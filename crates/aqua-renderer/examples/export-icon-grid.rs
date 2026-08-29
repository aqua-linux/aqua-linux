use aqua_renderer::icons::{composite_icon, IconRasterCache, IconRasterKey, IconRole, IconState};
use aqua_shell::AquaTheme;
use aqua_text::OutputScale;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("export-icon-grid requires an output PNG path")?;
    let cell = 64_u32;
    let width = cell * IconRole::ALL.len() as u32;
    let height = cell * AquaTheme::ALL.len() as u32;
    let mut rgba = vec![0_u8; (width * height * 4) as usize];
    let mut cache = IconRasterCache::default();

    for (theme_index, theme) in AquaTheme::ALL.into_iter().enumerate() {
        let background = match theme {
            AquaTheme::LightWhite => [0xf5, 0xf9, 0xfe, 0xff],
            AquaTheme::Softtouch => [0xe5, 0xee, 0xf5, 0xff],
            AquaTheme::Deepside => [0x18, 0x3a, 0x5d, 0xff],
            AquaTheme::Nightmare => [0x1f, 0x24, 0x2b, 0xff],
        };
        for y in theme_index as u32 * cell..(theme_index as u32 + 1) * cell {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&background);
            }
        }
        for (role_index, role) in IconRole::ALL.into_iter().enumerate() {
            let key = IconRasterKey::new(role, theme, IconState::Normal, 48, OutputScale::One)?;
            let icon = cache.get_or_render(key)?;
            composite_icon(
                &mut rgba,
                width,
                height,
                role_index as u32 * cell + 8,
                theme_index as u32 * cell + 8,
                &icon,
            );
        }
    }

    let file = File::create(output)?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&rgba)?;
    Ok(())
}
