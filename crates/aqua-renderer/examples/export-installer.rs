use aqua_installer::{
    DiskIdentity, InstallMode, InstallTarget, InstallerFormKey, InstallerFormState, InstallerModel,
    InstallerSummaryKey, InstallerUiState, InstallerUserFormKey,
};
use aqua_renderer::{
    export_installer_window_png_with_theme, InstallerImageSource, InstallerRenderOptions,
};
use aqua_shell::AquaTheme;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let logo_path = repository.join("docs/aqua-linux/assets/aqua-symbol-primary.png");
    let mut arguments = std::env::args_os().skip(1);
    let output = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| repository.join("build/installer-welcome.png"));
    let requested_step = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "welcome".to_string());
    let theme = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .map(|value| AquaTheme::parse(&value).ok_or("invalid Aqua theme"))
        .transpose()?
        .unwrap_or_default();
    let (logo_width, logo_height, logo_rgba) = decode_png_rgba(&logo_path)?;
    let logo = InstallerImageSource::new(logo_width, logo_height, &logo_rgba)?;
    let mut model = InstallerModel::default();
    let mut forms = InstallerFormState::default();
    match requested_step.as_str() {
        "welcome" => {}
        "language" => prepare_language(&mut model, &mut forms)?,
        "keyboard" => prepare_keyboard(&mut model, &mut forms)?,
        "partitions" => prepare_partitions(&mut model, &mut forms)?,
        "time-zone" => prepare_time_zone(&mut model, &mut forms)?,
        "user-information" => prepare_user_information(&mut model, &mut forms)?,
        "summary" => {
            prepare_user_information(&mut model, &mut forms)?;
            model.advance()?;
            model.set_mode(InstallMode::Real);
            forms
                .summary_mut()
                .handle_key(&mut model, InstallerSummaryKey::Activate)?;
            for character in model.confirmation_phrase().unwrap().chars() {
                forms
                    .summary_mut()
                    .handle_key(&mut model, InstallerSummaryKey::Character(character))?;
            }
            forms
                .summary_mut()
                .handle_key(&mut model, InstallerSummaryKey::Activate)?;
        }
        other => return Err(format!("unsupported installer export step: {other}").into()),
    }
    let ui = InstallerUiState::new(&model);
    let (png, probe) = export_installer_window_png_with_theme(
        1280,
        800,
        &model,
        &ui,
        &forms,
        logo,
        InstallerRenderOptions {
            progress: None,
            theme,
        },
    )?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(&output)?.write_all(&png)?;
    println!("installer_rendered={}", probe.rendered);
    println!("installer_layout_valid={}", probe.layout_valid);
    println!("installer_step={}", probe.step.id());
    println!("installer_focus={}", probe.focus.id());
    println!("installer_theme={}", theme.id());
    println!("installer_logo_rendered={}", probe.logo_rendered);
    println!("installer_primitive_count={}", probe.primitive_count);
    println!("installer_checksum={:016x}", probe.checksum);
    println!("installer_png={}", output.display());
    Ok(())
}

fn prepare_language(
    model: &mut InstallerModel,
    forms: &mut InstallerFormState,
) -> Result<(), Box<dyn Error>> {
    model.advance()?;
    forms.handle_key(model, InstallerFormKey::Activate)?;
    Ok(())
}

fn prepare_keyboard(
    model: &mut InstallerModel,
    forms: &mut InstallerFormState,
) -> Result<(), Box<dyn Error>> {
    prepare_language(model, forms)?;
    model.advance()?;
    forms.handle_key(model, InstallerFormKey::Activate)?;
    Ok(())
}

fn prepare_partitions(
    model: &mut InstallerModel,
    forms: &mut InstallerFormState,
) -> Result<(), Box<dyn Error>> {
    prepare_keyboard(model, forms)?;
    model.advance()?;
    let target = InstallTarget::erase_disk(DiskIdentity::new(
        "/dev/vdb",
        "qemu-aqua-installer-preview",
        "QEMU HARDDISK",
        32 * 1024 * 1024 * 1024,
    )?);
    forms.load_selected_target(&target);
    forms.handle_disk_key(model, InstallerFormKey::Activate)?;
    Ok(())
}

fn prepare_time_zone(
    model: &mut InstallerModel,
    forms: &mut InstallerFormState,
) -> Result<(), Box<dyn Error>> {
    prepare_partitions(model, forms)?;
    model.advance()?;
    forms.handle_key(model, InstallerFormKey::Activate)?;
    Ok(())
}

fn prepare_user_information(
    model: &mut InstallerModel,
    forms: &mut InstallerFormState,
) -> Result<(), Box<dyn Error>> {
    prepare_time_zone(model, forms)?;
    model.advance()?;
    for character in "aqua".chars() {
        forms
            .user_mut()
            .handle_key(model, InstallerUserFormKey::Character(character))?;
    }
    forms
        .user_mut()
        .handle_key(model, InstallerUserFormKey::NextField)?;
    for character in "Aqua Kullanıcısı".chars() {
        forms
            .user_mut()
            .handle_key(model, InstallerUserFormKey::Character(character))?;
    }
    forms
        .user_mut()
        .handle_key(model, InstallerUserFormKey::NextField)?;
    forms
        .user_mut()
        .handle_key(model, InstallerUserFormKey::SetPasswordConfigured(true))?;
    forms
        .user_mut()
        .handle_key(model, InstallerUserFormKey::Activate)?;
    Ok(())
}

fn decode_png_rgba(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
    let decoder = png::Decoder::new(BufReader::new(File::open(path)?));
    let mut reader = decoder.read_info()?;
    let output_size = reader
        .output_buffer_size()
        .ok_or("installer logo output buffer is too large")?;
    let mut decoded = vec![0; output_size];
    let info = reader.next_frame(&mut decoded)?;
    if info.bit_depth != png::BitDepth::Eight {
        return Err("installer logo must use 8-bit channels".into());
    }
    let bytes = &decoded[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xff])
            .collect(),
        png::ColorType::Rgba => bytes.to_vec(),
        other => return Err(format!("unsupported installer logo color type: {other:?}").into()),
    };
    Ok((info.width, info.height, rgba))
}
