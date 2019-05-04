extern crate quicli;
extern crate structopt;
extern crate walkdir;

use quicli::prelude::*;
use std::path::Path;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
struct Cli {
    // The absolute filepath you want to import all mp3s
    track_source: String,

    #[structopt(flatten)]
    verbosity: Verbosity,
}


/*
    Example usege
    ---
    - Get default track info (artist, album, track title, year): `tmt ./song.mp3`
    - Get artist on track: `tmt ./song.mp3 --artist`
    - Get artist and album on track: `tmt ./song.mp3 --artist --album`
    - Set artist on track: `tmt ./song.mp3 --artist=Fugazi`
    - Set artist and album on track: `tmt ./song.mp3 --artist="Fugazi" --album="13 Songs"`

    All Sets for a directory should confirm "are you sure you want the following settings for all
    mp3 tracks in the following directory {}?"
    - Get default track info for each in directory (artist, album, track title, year): `tmt ./Fugazi`
    - Get artist on each track in directory: `tmt ./Fugazi --artist`
    - Get artist and album on track in directory: `tmt ./Fugazi --artist --album`
    - Set artist on track: `tmt ./Fugazi --artist=Fugazi`
    - Set artist and album on track: `tmt ./Fugazi --artist="Fugazi" --album="13 Songs"`
*/

fn main() -> CliResult {
    let args = Cli::from_args();

    if !Path::new(&args.track_source).exists() {
        warn!("Error: {:?} is not a valid path", args.track_source);
    }

    let mut file_count = 0.0;
    let mp3_files = get_all_files_in_directory(&args.track_source);
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
