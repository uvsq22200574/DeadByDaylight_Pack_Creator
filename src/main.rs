use colored::Colorize;
use image::imageops::overlay;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod helper;

type SettingsMap = HashMap<String, String>; // key = element type, value = layer folder
type GameFolders = HashMap<String, HashMap<String, Vec<String>>>;

/// Run the `main` function to start reading the `SourcePack` folder and apply layers based
/// on the element type. It will look through the folders in the settings.json file and place it in `Output_Pack`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load settings.json
    let settings_file = File::open("settings.json")?;
    let settings: SettingsMap = serde_json::from_reader(settings_file)?;

    // Load elements_layering.json
    let file = File::open("elements_layering.json")?;
    let data: GameFolders = serde_json::from_reader(file)?;

    // Define and create the output
    let output_folder = "Output_Pack";
    std::fs::create_dir_all(output_folder)?;

    let mut current_image = 0;
    let mut skipped_images: Vec<String> = Vec::new();
    let total_images: usize = data.values().map(|elements| elements.len()).sum();

    for (element_type, elements) in &data {
        let layer_folder = match settings.get(element_type) {
            Some(folder) => folder,
            None => {
                std::io::stdout().flush().unwrap();
                eprintln!(
                    "{}",
                    format!(
                        "Skipping folder '{}': no entry found in settings.json",
                        element_type
                    )
                    .yellow()
                );
                continue;
            }
        };

        for (filename, layers) in elements {
            current_image += 1;
            let percentage = (current_image as f64 / total_images as f64) * 100.0;

            if (current_image % 25 == 0) || (current_image == total_images) {
                let msg = format!(
                    "Processing image({}) #{} / {} ({:.2}%)",
                    element_type, current_image, total_images, percentage
                );
                // Clear the line and print progress
                print!("\r\x1b[2K\r{}", msg);
                std::io::stdout().flush().unwrap();
            }

            // Prepare output path
            let element_folder_name = Path::new(element_type)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"));
            let output_path = Path::new(output_folder)
                .join(element_folder_name)
                .join(format!("{filename}.png"));
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Load base item
            let source_folder = Path::new("SourcePack").join(element_type);
            let item_img_path = helper::force_png_path(&source_folder, filename);

            let item_img = match image::open(&item_img_path) {
                Ok(img) => img,
                Err(_) => {
                    skipped_images.push(filename.to_string());
                    // Flush progress line, print skipped file in red
                    std::io::stdout().flush().unwrap();
                    eprintln!(
                        "{}",
                        format!(
                            "Skipping file '{}': could not open '{}'",
                            filename,
                            item_img_path.display()
                        )
                        .red()
                    );
                    // Reprint progress
                    print!(
                        "\r\x1b[2K\rProcessing image({}) #{} / {} ({:.2}%)",
                        element_type, current_image, total_images, percentage
                    );
                    std::io::stdout().flush().unwrap();
                    continue;
                }
            };

            let mut final_img = image::DynamicImage::new_rgba8(item_img.width(), item_img.height());

            // Stack layers using the associated layer folder
            if !layer_folder.is_empty() {
                helper::stack_layers(&mut final_img, Path::new(layer_folder), layers);
            }

            // Overlay base icon
            overlay(&mut final_img, &item_img, 0, 0);

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
