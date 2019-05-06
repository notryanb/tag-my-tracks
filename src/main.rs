extern crate id3;
extern crate quicli;
extern crate structopt;
extern crate walkdir;

use id3::{Tag, Version};
use quicli::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(subcommand)]
    cmd: Command,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "read")]
    Read {
        #[structopt(long="artist")]
        artist: bool,

        #[structopt(long="album")]
        album: bool,

        #[structopt(long="year")]
        year: bool,
    },

    #[structopt(name = "write")]
    Write {
        #[structopt(long="artist")]
        artist: Option<String>,

        #[structopt(long="album")]
        album: Option<String>,

        #[structopt(long="year")]
        year: Option<i32>,
    },
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
    let path = &args.path;

    if !path.exists() {
        warn!("Error: {:?} is not a valid path", &path);
    }

    if path.is_file() {
        process_file(&args, &path);
        // let mut tag = Tag::read_from_path(&path).unwrap();

        // match &args.cmd {
        //     Command::Read { artist, album, year } => {
        //         if *artist {
        //             println!("Artist: {}", tag.artist().unwrap());
        //         }
        //         if *album {
        //             println!("Album: {}", tag.album().unwrap());
        //         }
        //         if *year {
        //             println!("Year: {}", tag.year().unwrap());
        //         }

        //     },
        //     Command::Write { artist, album, year} => {
        //         if artist.is_some() {
        //             tag.set_artist(artist.clone().unwrap());
        //         }

        //         if album.is_some() {
        //             tag.set_album(album.clone().unwrap());
        //         }

        //         if year.is_some() {
        //             tag.set_year(year.unwrap());
        //         }


        //         tag.write_to_path(&path, Version::Id3v24)?;
        //     }
        // }


    } else {
        let mp3_files = get_all_files_in_directory(&args.path);
        for file in mp3_files.into_iter() {
            let path = PathBuf::from(file);
            process_file(&args, &path);
        }
    }


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

pub fn get_all_files_in_directory(directory: &PathBuf) -> Vec<String> {
    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| get_mp3_file_paths(&e))
        .collect()
}

pub fn process_file(args: &Cli, path: &PathBuf) { 
    let mut tag = Tag::read_from_path(&path).unwrap();
    
    match &args.cmd {
        Command::Read { artist, album, year } => {
            if *artist {
                println!("Artist: {}", tag.artist().unwrap());
            }
            if *album {
                println!("Album: {}", tag.album().unwrap());
            }
            if *year {
                println!("Year: {}", tag.year().unwrap());
            }

        },
        Command::Write { artist, album, year} => {
            if artist.is_some() { 
                tag.set_artist(artist.clone().unwrap());
            }

            if album.is_some() {
                tag.set_album(album.clone().unwrap());
            }

            if year.is_some() {
                tag.set_year(year.unwrap());
            }
            

            tag.write_to_path(&path, Version::Id3v24).unwrap();
        }
    }
}
