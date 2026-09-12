use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::ExtendedColorType;

pub const ICON_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

/// Je fournis à Windows chaque résolution usuelle à partir du même logo.
pub fn encode_windows_icon(source: &[u8]) -> Result<Vec<u8>, String> {
    let frames = ICON_SIZES
        .into_iter()
        .map(|size| {
            let canvas = crate::icon_asset::square_icon(source, size)?;
            IcoFrame::as_png(canvas.as_raw(), size, size, ExtendedColorType::Rgba8)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut bytes = Vec::new();
    IcoEncoder::new(&mut bytes)
        .encode_images(&frames)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}
