#![forbid(unsafe_code)]

#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::error::Error;
#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::io::BufWriter;
#[cfg(windows)]
use std::path::{Path, PathBuf};

#[cfg(windows)]
use image::codecs::ico::{IcoEncoder, IcoFrame};
#[cfg(windows)]
use image::imageops::FilterType;
#[cfg(windows)]
use image::{DynamicImage, ExtendedColorType, Rgba, RgbaImage};

#[cfg(windows)]
fn main() {
    println!("cargo:rerun-if-changed=icone/icone.png");
    configure_windows_subsystem();

    if let Err(error) = compile_windows_resources() {
        panic!("failed to compile Windows resources: {error}");
    }
}

#[cfg(not(windows))]
fn main() {
    println!("cargo:rerun-if-changed=icone/icone.png");
}

#[cfg(windows)]
fn compile_windows_resources() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let icon_path = manifest_dir.join("icone").join("icone.png");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let ico_path = out_dir.join("evofarm.ico");

    create_ico(&icon_path, &ico_path)?;

    let mut resource = winres::WindowsResource::new();
    let ico_path = ico_path.to_string_lossy().into_owned();
    resource.set_icon(&ico_path);
    resource.compile()?;

    Ok(())
}

#[cfg(windows)]
fn configure_windows_subsystem() {
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match target_env.as_str() {
        "msvc" => println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS"),
        "gnu" => println!("cargo:rustc-link-arg-bins=-mwindows"),
        _ => {}
    }
}

#[cfg(windows)]
fn create_ico(source_path: &Path, target_path: &Path) -> Result<(), Box<dyn Error>> {
    let image = image::open(source_path)?.into_rgba8();
    let resized = DynamicImage::ImageRgba8(image)
        .resize(256, 256, FilterType::Lanczos3)
        .to_rgba8();

    let mut canvas = RgbaImage::from_pixel(256, 256, Rgba([0, 0, 0, 0]));
    let offset_x = i64::from((256 - resized.width()) / 2);
    let offset_y = i64::from((256 - resized.height()) / 2);
    image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);

    let file = File::create(target_path)?;
    let writer = BufWriter::new(file);
    let frame = IcoFrame::as_png(
        canvas.as_raw(),
        canvas.width(),
        canvas.height(),
        ExtendedColorType::Rgba8,
    )?;

    IcoEncoder::new(writer).encode_images(&[frame])?;

    Ok(())
}
