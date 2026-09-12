use image::{imageops::FilterType, Rgba, RgbaImage};

/// Je centre le logo dans une icône carrée en conservant ses proportions et son alpha.
pub fn square_icon(source: &[u8], size: u32) -> Result<RgbaImage, String> {
    if size == 0 || size > 256 {
        return Err("La taille de l’icône doit être comprise entre 1 et 256 pixels.".into());
    }

    let image = image::load_from_memory(source).map_err(|error| error.to_string())?;
    let resized = image.resize(size, size, FilterType::Lanczos3).to_rgba8();
    let mut canvas = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    let offset_x = i64::from((size - resized.width()) / 2);
    let offset_y = i64::from((size - resized.height()) / 2);
    image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);
    Ok(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat};
    use std::io::Cursor;

    fn png(image: RgbaImage) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn landscape_logo_keeps_its_proportions_and_transparent_padding() {
        let source = png(RgbaImage::from_pixel(80, 40, Rgba([20, 150, 210, 255])));
        let icon = square_icon(&source, 32).unwrap();

        assert_eq!(icon.dimensions(), (32, 32));
        assert!(icon.rows().take(8).flatten().all(|pixel| pixel[3] == 0));
        assert!(icon.rows().skip(24).flatten().all(|pixel| pixel[3] == 0));
        assert!(icon
            .rows()
            .skip(8)
            .take(16)
            .flatten()
            .all(|pixel| *pixel == Rgba([20, 150, 210, 255])));
    }

    #[test]
    fn portrait_logo_keeps_its_proportions_and_source_transparency() {
        let source = png(RgbaImage::from_pixel(20, 40, Rgba([20, 150, 210, 128])));
        let icon = square_icon(&source, 32).unwrap();

        for y in 0..32 {
            for x in 0..32 {
                let expected_alpha = if (8..24).contains(&x) { 128 } else { 0 };
                assert_eq!(icon.get_pixel(x, y)[3], expected_alpha);
            }
        }
    }

    #[test]
    fn invalid_logo_and_invalid_size_are_reported() {
        assert!(square_icon(b"not a png", 32).is_err());
        assert!(square_icon(include_bytes!("../logo.png"), 0).is_err());
        assert!(square_icon(include_bytes!("../logo.png"), 257).is_err());
    }

    #[test]
    fn supplied_logo_produces_an_rgba_window_icon() {
        let icon = square_icon(include_bytes!("../logo.png"), 256).unwrap();
        assert_eq!(icon.dimensions(), (256, 256));
        assert_eq!(icon.as_raw().len(), 256 * 256 * 4);
        assert!(icon.pixels().any(|pixel| pixel[3] > 0));
    }
}
