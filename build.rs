#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::error::Error;
#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
#[path = "src/icon_asset.rs"]
mod icon_asset;
#[cfg(windows)]
#[path = "build_support/windows_icon.rs"]
mod windows_icon;

#[cfg(windows)]
fn main() {
    track_resource_inputs();
    configure_windows_subsystem();

    if let Err(error) = compile_windows_resources() {
        panic!("Impossible de compiler les ressources Windows d’EvoFarm : {error}");
    }
}

#[cfg(not(windows))]
fn main() {
    track_resource_inputs();
}

fn track_resource_inputs() {
    for path in [
        "logo.png",
        "Cargo.toml",
        "build.rs",
        "src/icon_asset.rs",
        "build_support/windows_icon.rs",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
}

#[cfg(windows)]
fn compile_windows_resources() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let icon_path = manifest_dir.join("logo.png");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let ico_path = out_dir.join("evofarm.ico");

    let source = std::fs::read(icon_path)?;
    let icon = windows_icon::encode_windows_icon(&source)?;
    std::fs::write(&ico_path, icon)?;

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
