use colored::Colorize;
use image::imageops::overlay;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod helper;

type GameFolders = HashMap<String, HashMap<String, Vec<String>>>;

#[derive(serde::Deserialize)]
struct Settings {
    layers_location: HashMap<String, String>,
    output_path: Option<String>,
    input_path: Option<String>,
}

struct Task {
    element_type: String,
    filename: String,
    layers: Vec<String>,
    layer_folder: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Measure processing time
    let start_time = Instant::now();

    // Load settings.json
    let settings_file = File::open("settings.json")?;
    let settings: Settings = serde_json::from_reader(settings_file)?;

    let platform = helper::detect_platform();
    println!("{}", format!("Platform: {:?}", platform).yellow());

    // Resolve input folder (use default if missing or empty)
    let source_folder = helper::resolve_or_default(
        settings.input_path.as_deref(),
        Path::new("Source_Pack"),
        platform,
    );

    // Check if the input folder exists, else return an error
    if !source_folder.exists() || !source_folder.is_dir() {
        return Err(format!(
            "Input folder does not exist or is not a directory: {}",
            source_folder.display()
        )
        .into());
    }

    println!("{}", format!("Input folder: {}", source_folder.display()).yellow());

    // Resolve output folder (use default if missing or empty)
    let output_folder = helper::resolve_or_default(
        settings.output_path.as_deref(),
        Path::new("Output_Pack"),
        platform,
    );
    let output_folder = helper::resolve_full_path(&output_folder);
    std::fs::create_dir_all(&output_folder)?;
    println!("{}", format!("Output folder: {}", output_folder.display()).yellow());

    // Load elements_layering.json
    let file = File::open("elements_layering.json")?;
    let data: GameFolders = serde_json::from_reader(file)?;

    // Collect tasks
    let mut tasks = Vec::new();
    for (element_type, elements) in &data {
        let layer_folder_path = settings
            .layers_location
            .get(element_type)
            .filter(|s| !s.is_empty())
            .map(|s| {
                let p = helper::resolve_full_path(&PathBuf::from(s));
                if helper::is_path_compatible(&p, platform) && p.exists() {
                    p
                } else {
                    PathBuf::new()
                }
            })
            .unwrap_or_else(PathBuf::new);
        let layer_folder_path = helper::resolve_full_path(&layer_folder_path);
        println!(
            "{}",
            format!("Layer folder for '{}': {}", element_type, layer_folder_path.display())
                .yellow()
        );

        for (filename, layers) in elements {
            tasks.push(Task {
                element_type: element_type.clone(),
                filename: filename.clone(),
                layers: layers.clone(),
                layer_folder: layer_folder_path.clone(),
            });
        }
    }

    let skipped_images = Arc::new(Mutex::new(Vec::new()));
    let missing_layers = Arc::new(Mutex::new(Vec::new()));

    // Process images in parallel
    tasks.par_iter().for_each(|task| {
        let Task {
            element_type,
            filename,
            layers,
            layer_folder,
        } = task;

        let item_img_path = helper::force_png_path(&source_folder.join(element_type), filename);

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

        if !layer_folder.as_os_str().is_empty() {
            let missing = helper::stack_layers(&mut final_img, &item_img_path, &layer_folder, layers);
            let mut missing_lock = missing_layers.lock().unwrap();
            missing_lock.extend(missing);
        }

        overlay(&mut final_img, &item_img, 0, 0);

        let element_folder_name = Path::new(element_type)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"));
        let output_path = output_folder.join(element_folder_name).join(format!("{filename}.png"));

        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Err(e) = final_img.save(&output_path) {
            eprintln!("Failed to save '{}': {}", output_path.display(), e);
        }
    });

    println!("\n{}", "Processing complete!".green());

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

    let elapsed = start_time.elapsed();
    println!("{}",format!("Total processing time: {:.2?}", elapsed).cyan());

    Ok(())
}
