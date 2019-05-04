extern crate quicli;
extern crate structopt;
extern crate walkdir;

use quicli::prelude::*;
use std::path::Path;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

/* 
    TODO:

*/

#[derive(Debug, StructOpt)]
struct Cli {
    // The absolute filepath you want to import all mp3s
    directory: String,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();

    if !Path::new(&args.directory).exists() {
        warn!("Error: {:?} is not a valid path", args.directory);
    }

    let mut file_count = 0.0;
    let mp3_files = get_all_files_in_directory(&args.directory);
    let total_files = mp3_files.clone().into_iter().count() as f32;
    
    println!("Total Files: {}", &total_files);

    for e in mp3_files.into_iter() {
        let progress = ((file_count / total_files) * 100.0).round();
        println!(
            "Count: {}, Progress: {}%, File: {:?}",
            &file_count, progress, &e
        );
        file_count += 1.0;
    };

    Ok(())
}

/// Returns the file path if it's a .mp3 file or None.
pub fn get_mp3_file_paths(entry: &DirEntry) -> Option<String> {
    match entry.path().extension() {
        Some(ext) => match ext.to_str() {
            Some(exxt) if exxt == "mp3" => match entry.path().to_str() {
                Some(p) => Some(p.to_string()),
                None => None,
            },
            Some(_) => None,
            None => None,
        },
        None => None,
    }
}

pub fn get_all_files_in_directory(directory: &String) -> Vec<String> {
    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| get_mp3_file_paths(&e))
        .collect()
}
