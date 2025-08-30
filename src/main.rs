use colored::*;
use image::imageops::overlay;
use phf::phf_map;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod helper;

/// Add the folders you need to this static data, the element type referencing the JSON and of course the `GameFolders` JSON itself
static ELEMENT_FOLDER_MAPPING: phf::Map<&'static str, &'static str> = phf_map! {
    "items" => "SourcePack/Items",
    "addons" => "SourcePack/ItemAddons",
    "powers" => "SourcePack/Powers",
    "offerings" => "SourcePack/Favors",
    "perks" => "SourcePack/Perks",
    "actions" => "SourcePack/Actions",
    "character_portraits" => "SourcePack/CharPortraits",
    "loading_screen" => "SourcePack/HelpLoading",
    "status_effects" => "SourcePack/StatusEffects",
    "archive" => "SourcePack/Archive",
    "emblems" => "SourcePack/Emblems",
};
/// Run the `main` function to start reading the `SourcePack` folder and apply layers based
/// on the element type. It will look through the (hardcoded) folders and place it in `OutPack`.
/// The binary should have a relative access to these folders: `SourcePack` `Layers`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("elements_layering.json")?;
    let data: helper::GameFolders = serde_json::from_reader(file)?;
    let element_types: &[(&str, &HashMap<String, Vec<String>>)] = &[
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

    let output_folder = "Output_Pack";
    std::fs::create_dir_all(output_folder)?;

    let mut current_image = 0;
    let mut skipped_images: Vec<String> = Vec::new();
    let total_images: usize = element_types
        .iter()
        .map(|(_, elements)| elements.len())
        .sum();

    for (element_type, elements) in element_types {
        let element_folder = match ELEMENT_FOLDER_MAPPING.get(element_type) {
            Some(p) => p,
            None => continue,
        };

        for (filename, layers) in *elements {
            current_image += 1;
            let percentage = (current_image as f64 / total_images as f64) * 100.0;
            if (current_image % 25 == 0) || (current_image == total_images) {
                print!(
                "\rProcessing image #{} / {} ({:.2}%)",
                current_image, total_images, percentage
            );
            std::io::stdout().flush().unwrap();
            }
            

            // Prepare output path
            let element_folder_name = Path::new(element_folder)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"));
            let output_path = Path::new(output_folder)
                .join(element_folder_name)
                .join(format!("{filename}.png"));
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Try to load the base item image
            let item_img_path = helper::force_png_path(Path::new(element_folder), filename);
            let item_img = match image::open(&item_img_path) {
                Ok(img) => img,
                Err(_) => {
                    skipped_images.push(filename.clone());
                    continue;
                }
            };

            // Defines the starting image output
            let mut final_img = image::DynamicImage::new_rgba8(item_img.width(), item_img.height());

            helper::stack_layers(&mut final_img, element_type, layers);

            // Overlay the main icon on top, regardless of layers (equivalent to just copying it)
            overlay(&mut final_img, &item_img, 0, 0);

            // Save the composed image
            if let Err(e) = final_img.save(&output_path) {
                eprintln!("Failed to save '{}': {}", output_path.display(), e);
            }
        }
    }

    println!("\n{}", "Processing complete!".green());

    if !skipped_images.is_empty() {
        println!("{}", "Skipped images:".red());
        for s in skipped_images {
            println!(" - {}", s);
        }
    }

    Ok(())
}
