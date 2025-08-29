use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use colored::*;
use image::imageops::overlay;
use serde::Deserialize;

mod color_image_mask;
use color_image_mask::colorize_grayscale_image;

#[derive(Deserialize)]
struct JsonData {
    #[serde(rename = "Actions")]
    actions: HashMap<String, Vec<String>>,
    #[serde(rename = "CharPortraits")]
    character_portraits: HashMap<String, Vec<String>>,
    #[serde(rename = "Archive")]
    archive: HashMap<String, Vec<String>>,
    #[serde(rename = "Favors")]
    offerings: HashMap<String, Vec<String>>,
    #[serde(rename = "HelpLoading")]
    loading_screen: HashMap<String, Vec<String>>,
    #[serde(rename = "ItemAddons")]
    addons: HashMap<String, Vec<String>>,
    #[serde(rename = "Items")]
    items: HashMap<String, Vec<String>>,
    #[serde(rename = "Perks")]
    perks: HashMap<String, Vec<String>>,
    #[serde(rename = "Powers")]
    powers: HashMap<String, Vec<String>>,
    #[serde(rename = "StatusEffects")]
    status_effects: HashMap<String, Vec<String>>,
    #[serde(rename = "Emblems")]
    emblems: HashMap<String, Vec<String>>,
}

/// Normalize path to `.png`
fn force_png_path(base: &Path, name: &str) -> PathBuf {
    base.join(format!("{}.png", name))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("elements_layering.json")?;
    let data: JsonData = serde_json::from_reader(file)?;

    let background_base_folder = "Rarities";

    let background_type_folders: HashMap<&str, &str> = HashMap::from([
        ("items", "addons-items-powers"),
        ("addons", "addons-items-powers"),
        ("powers", "addons-items-powers"),
        ("offerings", "offerings"),
        ("perks", "perks"),
    ]);

    let pack_type_folders: HashMap<&str, &str> = HashMap::from([
        ("items", "SourcePack/Items"),
        ("addons", "SourcePack/ItemAddons"),
        ("powers", "SourcePack/Powers"),
        ("offerings", "SourcePack/Favors"),
        ("perks", "SourcePack/Perks"),
        ("actions", "SourcePack/Actions"),
        ("character_portraits", "SourcePack/CharPortraits"),
        ("loading_screen", "SourcePack/HelpLoading"),
        ("status_effects", "SourcePack/StatusEffects"),
        ("archive", "SourcePack/Archive"),
        ("emblems", "SourcePack/Emblems"),
    ]);

    let output_folder = "Output_Pack";
    std::fs::create_dir_all(output_folder)?;

    let element_types: Vec<(&str, &HashMap<String, Vec<String>>)> = vec![
        ("items", &data.items),
        ("addons", &data.addons),
        ("powers", &data.powers),
        ("offerings", &data.offerings),
        ("perks", &data.perks),
        ("actions", &data.actions),
        ("character_portraits", &data.character_portraits),
        ("loading_screen", &data.loading_screen),
        ("status_effects", &data.status_effects),
        ("archive", &data.archive),
        ("emblems", &data.emblems),
    ];

    let total_images: usize = element_types.iter().map(|(_, elements)| elements.len()).sum();
    let mut current_image = 0;

    // Collect skipped images in case of errors or missing files
    let mut skipped_images: Vec<String> = Vec::new();

    for (element_type, elements) in element_types {
        let pack_folder = match pack_type_folders.get(element_type) {
            Some(p) => p,
            None => {
                eprintln!("No source folder defined for '{}', skipping", element_type);
                continue;
            }
        };

        let rarity_folder_opt = background_type_folders.get(element_type);

        for (filename, layers) in elements {
            current_image += 1;
            let percentage = (current_image as f64 / total_images as f64) * 100.0;
            print!(
                "\rProcessing image #{} / {} ({:.2}%)",
                current_image, total_images, percentage
            );
            std::io::stdout().flush().unwrap();

            // Output path early for saving
            let pack_folder_name = Path::new(pack_folder)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"));
            let output_path = Path::new(output_folder)
                .join(pack_folder_name)
                .join(format!("{filename}.png"));
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let item_img_path = force_png_path(Path::new(pack_folder), filename);
            let item_img = match image::open(&item_img_path) {
                Ok(img) => img,
                Err(_) => {
                    skipped_images.push(filename.clone());
                    continue;
                }
            };

            let no_layer = layers.is_empty() || layers[0] == "none";
            let mut final_img = if no_layer {
                item_img.clone()
            } else {
                image::DynamicImage::new_rgba8(item_img.width(), item_img.height())
            };

            for layer_name in layers {
                if layer_name.is_empty() || layer_name == "none" {
                    continue;
                }

                if let Some(rarity_folder) = rarity_folder_opt {
                    let layer_img_path =
                        force_png_path(&Path::new(background_base_folder).join(rarity_folder), layer_name);
                    match image::open(&layer_img_path) {
                        Ok(layer_img) => overlay(&mut final_img, &layer_img, 0, 0),
                        Err(_) => {} // silently ignore missing layer
                    }
                }
            }

            overlay(&mut final_img, &item_img, 0, 0);

            if let Err(e) = final_img.save(&output_path) {
                eprintln!("Failed to save '{}': {}", output_path.display(), e);
            }
        }
    }

    // Finish progress line
    println!("\n{}", "Processing complete!".green());

    // Print skipped images
    if !skipped_images.is_empty() {
        println!("{}", "Skipped images:".red());
        for s in skipped_images {
            println!(" - {}", s);
        }
        println!("{}", "-> You might want to check that the JSON do not contain deleted files or that any other error happened".magenta())
    }

    Ok(())
}
