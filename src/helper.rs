use image::{DynamicImage, ImageBuffer, Rgba, imageops::overlay};
use std::path::{Path, PathBuf};

/// Convert a hex string like "#RRGGBB" or "RRGGBB" into (r, g, b)
pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err("Hex color must be 6 characters long".to_string());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red value")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green value")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue value")?;

    Ok((r, g, b))
}

/// Useful to make a grayscale mask change color, preserving transparency
pub fn colorize_grayscale_image(
    gray_img: &ImageBuffer<image::LumaA<u8>, Vec<u8>>,
    hex_color: &str,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    let (r_tint, g_tint, b_tint) = hex_to_rgb(hex_color)?;

    Ok(ImageBuffer::from_fn(
        gray_img.width(),
        gray_img.height(),
        |x, y| {
            let gray_pixel = gray_img.get_pixel(x, y);
            let gray_val = gray_pixel[0];
            let alpha = gray_pixel[1];

            Rgba([
                (gray_val as u16 * r_tint as u16 / 255) as u8,
                (gray_val as u16 * g_tint as u16 / 255) as u8,
                (gray_val as u16 * b_tint as u16 / 255) as u8,
                alpha,
            ])
        },
    ))
}

/// Normalize path to `.png`
pub fn force_png_path(base: &Path, name: &str) -> PathBuf {
    base.join(format!("{}.png", name))
}

pub fn resolve_full_path(path: &Path) -> PathBuf {
    let mut p = path.to_path_buf();

    // Expand ~ on Unix-like systems
    #[cfg(unix)]
    if let Some(path_str) = path.to_str() {
        if path_str == "~" || path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                let stripped = path_str.trim_start_matches("~/");
                p = home.join(stripped);
            }
        }
    }

    // Convert to absolute if it's not already
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

/// Apply layers using the provided layer folder
/// Returns a list of missing layer file paths.
pub fn stack_layers(
    input_image: &mut DynamicImage,
    layer_folder: &Path,
    layers: &Vec<String>,
) -> Vec<String> {
    let mut missing_layers = Vec::new();

    for layer_name in layers {
        // Skip empty or "none"
        if layer_name.is_empty() || layer_name == "none" {
            continue;
        }

        // Split name and optional HEX color
        let mut parts = layer_name.splitn(2, '#');
        let base_name = parts.next().unwrap();
        let hex_color = parts.next();

        // Build the full path to the layer image
        let layer_img_path = force_png_path(layer_folder, base_name);

        // Try opening the layer image
        match image::open(&layer_img_path) {
            Ok(layer_img) => {
                let mut processed_img = layer_img.clone();

                // Recolor grayscale layer if HEX specified
                if let Some(hex) = hex_color {
                    let gray_img = layer_img.to_luma_alpha8();
                    if let Ok(colored) = colorize_grayscale_image(&gray_img, hex) {
                        processed_img = DynamicImage::ImageRgba8(colored);
                    }
                }

                // Overlay the layer on top of the input image
                overlay(input_image, &processed_img, 0, 0);
            }
            Err(_) => {
                // Accumulate missing layer paths instead of printing
                missing_layers.push(layer_img_path.display().to_string());
            }
        }
    }

    missing_layers
}
