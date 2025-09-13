use colored::Colorize;
use image::imageops::overlay;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod helper;

type SettingsMap = HashMap<String, String>;
type GameFolders = HashMap<String, HashMap<String, Vec<String>>>;

struct Task {
    element_type: String,
    filename: String,
    layers: Vec<String>,
    layer_folder: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load settings.json
    let settings_file = File::open("settings.json")?;
    let settings: SettingsMap = serde_json::from_reader(settings_file)?;

    // Load elements_layering.json
    let file = File::open("elements_layering.json")?;
    let data: GameFolders = serde_json::from_reader(file)?;

    let output_folder = "Output_Pack";
    std::fs::create_dir_all(output_folder)?;

    // Collect all tasks
    let mut tasks = Vec::new();
    for (element_type, elements) in &data {
        if let Some(folder) = settings.get(element_type) {
            for (filename, layers) in elements {
                tasks.push(Task {
                    element_type: element_type.clone(),
                    filename: filename.clone(),
                    layers: layers.clone(),
                    layer_folder: folder.clone(),
                });
            }
        } else {
            eprintln!(
                "{}",
                format!(
                    "Skipping folder '{}': no entry found in settings.json",
                    element_type
                )
                .yellow()
            );
        }
    }

    let skipped_images = Arc::new(Mutex::new(Vec::new()));
    let missing_layers = Arc::new(Mutex::new(Vec::new()));

    // Process images in parallel
    tasks.par_iter().enumerate().for_each(|(_, task)| {
        let Task {
            element_type,
            filename,
            layers,
            layer_folder,
        } = task;

        let source_folder = Path::new("SourcePack").join(element_type);
        let item_img_path = helper::force_png_path(&source_folder, filename);

        let item_img = match image::open(&item_img_path) {
            Ok(img) => img,
            Err(_) => {
                let mut skipped = skipped_images.lock().unwrap();
                skipped.push(filename.clone());
                eprintln!(
                    "{}",
                    format!(
                        "Skipping file '{}': could not open '{}'",
                        filename,
                        item_img_path.display()
                    )
                    .red()
                );
                return;
            }
        };

        let mut final_img = image::DynamicImage::new_rgba8(item_img.width(), item_img.height());

        if !layer_folder.is_empty() {
            let missing = helper::stack_layers(&mut final_img, Path::new(layer_folder), layers);
            let mut missing_lock = missing_layers.lock().unwrap();
            missing_lock.extend(missing);
        }

        overlay(&mut final_img, &item_img, 0, 0);

        let element_folder_name = Path::new(element_type)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"));
        let output_path = Path::new(output_folder)
            .join(element_folder_name)
            .join(format!("{filename}.png"));
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Err(e) = final_img.save(&output_path) {
            eprintln!("Failed to save '{}': {}", output_path.display(), e);
        }
    });

    println!("\n{}", "Processing complete!".green());

    // Print skipped files
    let skipped = skipped_images.lock().unwrap();
    if !skipped.is_empty() {
        println!("{}", "Skipped images:".red());
        for s in skipped.iter() {
            println!(" - {}", s);
        }
    }

    let missing = missing_layers.lock().unwrap();
    if !missing.is_empty() {
        println!("{}", "Skipped layers:".red());
        for s in missing.iter() {
            println!(" - {}", s);
        }
    }

    Ok(())
}
