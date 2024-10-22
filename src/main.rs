use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

#[derive(serde::Serialize)]
struct FileStructure {
    size: u64,                             // Size as a human-readable string
    files: HashMap<String, FileStructure>, // Nested files and folders
}

fn traverse_directory(path: &Path, depth: usize) -> io::Result<FileStructure> {
    if depth == 0 {
        return Ok(FileStructure {
            size: 0,
            files: HashMap::new(),
        });
    }

    let mut total_size: u64 = 0;
    let mut structure = FileStructure {
        size: 0,
        files: HashMap::new(),
    };

    // Use guard clause to handle error
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => {
            println!("Could not read directory: {:?}", path);
            return Ok(structure);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                println!("Could not access entry in directory: {:?}", path);
                continue;
            }
        };

        let name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| "Invalid UTF-8 Name".to_string());
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                println!("Could not determine file type: {:?}", entry.path());
                continue;
            }
        };

        if file_type.is_dir() {
            // Recursively traverse subdirectory
            let sub_structure = match traverse_directory(&entry.path(), depth - 1) {
                Ok(sub_structure) => sub_structure,
                Err(_) => {
                    println!("Could not access directory: {:?}", entry.path());
                    continue;
                }
            };

            total_size += sub_structure.size;
            structure.files.insert(name, sub_structure);
        } else {
            // Handle file size
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    println!("Could not access file metadata: {:?}", entry.path());
                    continue;
                }
            };

            let file_size = metadata.len();
            total_size += file_size;
            structure.files.insert(
                name,
                FileStructure {
                    size: file_size,
                    files: HashMap::new(),
                },
            );
        }
    }

    // Set the total size after traversing
    structure.size = total_size;

    Ok(structure)
}

fn main() -> io::Result<()> {
    let path = Path::new("D:/Code/"); // Replace with your folder path
    let max_depth = 5; // Configure your desired depth here
    println!("Scanning directory {:?}", path); // Notify user

    let start_scan_time = Instant::now();
    let structure = traverse_directory(path, max_depth)?;
    let scan_duration = start_scan_time.elapsed();

    let yaml_output = serde_yaml::to_string(&structure).unwrap(); // Use serde_yaml

    let start_write_time = Instant::now();
    let output_file_path = Path::new("output.yaml"); // Define the output file path
    let mut file = File::create(output_file_path)?; // Create the file
    file.write_all(yaml_output.as_bytes())?; // Write the YAML content to the file
    let write_duration = start_write_time.elapsed();

    // Print time taken for scanning and writing
    println!("Time taken to scan the directory: {:?}", scan_duration);
    println!("Time taken to write the YAML file: {:?}", write_duration);
    println!("YAML output written to {:?}", output_file_path); // Notify user

    Ok(())
}
