use image::{ImageBuffer, Rgba};

/// Useful to make a grayscale mask change color
pub fn colorize_grayscale_image(
    gray_img: &ImageBuffer<image::Luma<u8>, Vec<u8>>,
    color: (u8, u8, u8),
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (r_tint, g_tint, b_tint) = color;

    ImageBuffer::from_fn(gray_img.width(), gray_img.height(), |x, y| {
        let gray_pixel = gray_img.get_pixel(x, y)[0];
        Rgba([
            (gray_pixel as u16 * r_tint as u16 / 255) as u8,
            (gray_pixel as u16 * g_tint as u16 / 255) as u8,
            (gray_pixel as u16 * b_tint as u16 / 255) as u8,
            255, // fully opaque
        ])
    })
}
