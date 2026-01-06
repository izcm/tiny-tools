use std::env;

use exif::{DateTime, In, Tag, Value};
use std::fs::File;
use std::io::BufReader;

use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read CLI args (std::env::args)
    let args: Vec<String> = env::args().collect();
    let input_dir: &str = args.get(1).expect("pass a directory");

    let output_dir = PathBuf::from("sorted");

    println!("Sorting directory: {input_dir}");
    println!("Output directory: {}", output_dir.display());

    // Step 2: Walk files recursively (walkdir)
    let filename = String::from("042.JPG");
    let source_path = PathBuf::from(input_dir).join(&filename);

    // Step 3: Open image + read EXIF (exif DateTimeOriginal) + parse year & month
    let file = File::open(&source_path)?;
    let mut buf_reader = BufReader::new(&file);

    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut buf_reader)?;

    let mut year: Option<u16> = None;
    let mut month: Option<u8> = None;

    if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        match field.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(datetime) = DateTime::from_ascii(&vec[0]) {
                    year = Some(datetime.year as u16);
                    month = Some(datetime.month as u8);
                }
            }
            _ => {}
        }
    }

    // Step 4: Build path
    let target_dir = match (year, month) {
        (Some(y), Some(m)) => {
            let mut p = output_dir.clone();
            p.push(y.to_string());
            p.push(m.to_string());
            p
        }
        _ => {
            let mut p = output_dir.clone();
            p.push("unknown");
            p
        }
    };

    println!("Found original date!");
    println!("Moving this to: {}", target_dir.display());

    // Step 5: Create folders if missing (std::fs::create_dir_all)
    std::fs::create_dir_all(&target_dir)?;

    // Step 6: Copy file (std::fs::copy)
    match fs::copy(&source_path, {
        let mut p = target_dir.clone();
        p.push(&filename);
        p
    }) {
        Ok(_) => println!("✅ copied"),
        Err(e) => println!("❌ failed: {e}"),
    };

    // Step 7: Log + continue
    Ok(())
}
