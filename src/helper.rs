use image::{ImageBuffer, Rgba, imageops::overlay};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct GameFolders {
    #[serde(rename = "Actions")]
    pub actions: HashMap<String, Vec<String>>,
    #[serde(rename = "CharPortraits")]
    pub character_portraits: HashMap<String, Vec<String>>,
    #[serde(rename = "Archive")]
    pub archive: HashMap<String, Vec<String>>,
    #[serde(rename = "Favors")]
    pub offerings: HashMap<String, Vec<String>>,
    #[serde(rename = "HelpLoading")]
    pub loading_screen: HashMap<String, Vec<String>>,
    #[serde(rename = "ItemAddons")]
    pub addons: HashMap<String, Vec<String>>,
    #[serde(rename = "Items")]
    pub items: HashMap<String, Vec<String>>,
    #[serde(rename = "Perks")]
    pub perks: HashMap<String, Vec<String>>,
    #[serde(rename = "Powers")]
    pub powers: HashMap<String, Vec<String>>,
    #[serde(rename = "StatusEffects")]
    pub status_effects: HashMap<String, Vec<String>>,
    #[serde(rename = "Emblems")]
    pub emblems: HashMap<String, Vec<String>>,
}

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

// Apply layers
pub fn stack_layers(
    input_image: &mut image::DynamicImage,
    element_type: &str,
    layers: &Vec<String>,
) {
    let background_folder_mapping: HashMap<&str, &str> = HashMap::from([
        ("items", "addons_items_powers"),
        ("addons", "addons_items_powers"),
        ("powers", "addons_items_powers"),
        ("offerings", "offerings"),
        ("perks", "perks"),
    ]);

    let layer_location = background_folder_mapping.get(element_type);

    for layer_name in layers {
        // If there is no layer simply skip the processing step
        if layer_name.is_empty() || layer_name == "none" {
            continue;
        }

        // Split name and optional HEX color
        let mut parts = layer_name.splitn(2, '#');
        let base_name = parts.next().unwrap();
        let hex_color = parts.next();

        // If the image has a layer associated, get its path and normalize it
        if let Some(layer_folder) = layer_location {
            let layer_img_path = force_png_path(&Path::new("Layers").join(layer_folder), base_name);

            // If the layer could be opened
            if let Ok(layer_img) = image::open(&layer_img_path) {
                let mut processed_img = layer_img.clone();

                // If the color could be determined
                if let Some(hex) = hex_color {
                    let gray_img = layer_img.to_luma_alpha8();
                    if let Ok(colored) = colorize_grayscale_image(&gray_img, hex) {
                        processed_img = image::DynamicImage::ImageRgba8(colored);
                    }
                }

                // Stack the layer
                overlay(input_image, &processed_img, 0, 0);
            }
        }
    }
}
