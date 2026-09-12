#[path = "../src/icon_asset.rs"]
mod icon_asset;
#[path = "../build_support/windows_icon.rs"]
mod windows_icon;

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

#[test]
fn ico_contains_all_seven_decodable_rgba_sizes() {
    let icon = windows_icon::encode_windows_icon(include_bytes!("../logo.png")).unwrap();
    assert_eq!(&icon[..4], &[0, 0, 1, 0]);
    assert_eq!(u16::from_le_bytes([icon[4], icon[5]]), 7);

    for (index, size) in windows_icon::ICON_SIZES.into_iter().enumerate() {
        let entry = &icon[6 + index * 16..6 + (index + 1) * 16];
        let encoded_dimension = if size == 256 { 0 } else { size as u8 };
        assert_eq!((entry[0], entry[1]), (encoded_dimension, encoded_dimension));
        let length = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
        let decoded = image::load_from_memory(&icon[offset..offset + length]).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (size, size));
        assert_eq!(decoded.color(), image::ColorType::Rgba8);
        assert!(decoded.to_rgba8().pixels().any(|pixel| pixel[3] > 0));
    }
}

#[test]
fn ico_keeps_alpha_and_uncropped_non_square_logo_at_every_resolution() {
    let source = RgbaImage::from_pixel(20, 40, Rgba([25, 100, 200, 128]));
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(source)
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    let ico = windows_icon::encode_windows_icon(png.get_ref()).unwrap();

    for (index, size) in windows_icon::ICON_SIZES.into_iter().enumerate() {
        let entry = &ico[6 + index * 16..6 + (index + 1) * 16];
        let length = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
        let frame = image::load_from_memory(&ico[offset..offset + length])
            .unwrap()
            .to_rgba8();
        for (x, _, pixel) in frame.enumerate_pixels() {
            assert_eq!(
                pixel[3],
                if (size / 4..size * 3 / 4).contains(&x) {
                    128
                } else {
                    0
                }
            );
        }
    }
}

#[test]
fn invalid_source_does_not_produce_an_icon() {
    assert!(windows_icon::encode_windows_icon(b"invalid image").is_err());
}
