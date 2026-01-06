use std::env;

use exif::{DateTime, In, Tag, Value};
use std::fs::File;
use std::io::BufReader;

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use std::collections::HashMap;

fn month_to_quarter(month: u8) -> &'static str {
    match month {
        1 | 2 | 3 => "01 kvartal ⛄",    // Q1
        4 | 5 | 6 => "02 kvartal 🌺",    // Q2
        7 | 8 | 9 => "03 kvartal 🌅",    // Q3
        10 | 11 | 12 => "04 kvartal 🍁", // Q4
        _ => "unknown",
    }
}

fn print_separator() {
    println!("========================");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read CLI args (std::env::args)
    let args: Vec<String> = env::args().collect();
    let input_dir: &str = args.get(1).expect("pass a directory");

    let output_dir = PathBuf::from("sorted");

    // Header
    print_separator();
    println!("In Dir  : {input_dir}");
    println!("Out Dir : {}", output_dir.display());
    print_separator();

    fs::create_dir(&output_dir)?;

    let mut skipped: HashMap<String, u32> = HashMap::new();
    let mut moved: HashMap<String, u32> = HashMap::new();

    // Step 2: Walk files recursively (walkdir)
    for entry in WalkDir::new(&input_dir) {
        let entry = entry?;

        let filename = entry.file_name();
        let extension = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        let source_path = PathBuf::from(entry.path());

        if !entry.file_type().is_file() {
            continue;
        }

        // Step 3: Open image + read EXIF (exif DateTimeOriginal) + parse year & month
        let file = File::open(&source_path)?;
        let mut buf_reader = BufReader::new(&file);

        let exif_reader = exif::Reader::new();
        let exif = match exif_reader.read_from_container(&mut buf_reader) {
            Ok(e) => e,
            Err(_) => {
                // TODO: don't skip .MOV just move to /filmer / .MOV datestamp?
                skipped
                    .entry(String::from(extension))
                    .and_modify(|v| *v += 1)
                    .or_insert(1);

                continue;
            }
        };

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
                p.push(month_to_quarter(m));
                p
            }
            _ => {
                let mut p = output_dir.clone();
                p.push("dato_ukjent");
                p
            }
        };

        // Step 5: Create folders if missing (std::fs::create_dir_all)
        fs::create_dir_all(&target_dir)?;

        // Step 6: Copy file (std::fs::copy)
        let copy_result = fs::copy(&source_path, {
            let mut p = target_dir.clone();
            p.push(&filename);
            p
        });

        if let Err(e) = copy_result {
            println!("   ↳ ❌ copy failed: {e}");
        } else {
            moved
                .entry(String::from(extension))
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }
    }

    // Summary: skipped files
    if !skipped.is_empty() {
        println!("Skipped:");
        for (key, value) in &skipped {
            println!("  {} .{} files", value, key);
        }
        print_separator();
    }

    // Summary: moved files
    if !moved.is_empty() {
        println!("Moved:");
        for (key, value) in &moved {
            println!("  {} .{} files", value, key);
        }
        print_separator();
    }

    Ok(())
}
