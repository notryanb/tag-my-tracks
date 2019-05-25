extern crate id3;
extern crate quicli;
extern crate structopt;
extern crate walkdir;

use id3::{Tag, Version};
use quicli::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Command", about = "Read or write fields from the ID3 Tags for a given path.")]
pub enum Command {
    #[structopt(name = "read", help = "Reads the requested fields from the ID3 tag(s) specified in the path")]
    Read {
        #[structopt(long = "artist")]
        artist: bool,

        #[structopt(long = "album")]
        album: bool,

        #[structopt(long = "title")]
        title: bool,

        #[structopt(long = "year")]
        year: bool,
    },

    #[structopt(name = "write", help = "Writes the requested fields and their values to ID3v2.4 tag(s) specified in the path")]
    Write {
        #[structopt(long = "artist")]
        artist: Option<String>,

        #[structopt(long = "album")]
        album: Option<String>,

        #[structopt(long = "title")]
        title: Option<String>,

        #[structopt(long = "year")]
        year: Option<i32>,
    },
}

// TODO - Better error handling
fn main() -> CliResult {
    let args = Cli::from_args();
    let path = &args.path;

    if !path.exists() {
        warn!("Error: {:?} is not a valid path", &path);
    }

    if path.is_file() {
        process_file(&args, &path);
    } else {
        let mp3_paths = get_all_mp3_files_in_directory(&args.path);
        for path in mp3_paths.into_iter() {
            process_file(&args, &path);
        }
    }

    Ok(())
}

// Returns the file path if it's a .mp3 file or None.
pub fn mp3_file_paths(dir_entry: &DirEntry) -> Option<PathBuf> {
    let path = dir_entry.path();
    match &path.extension() {
        Some(extension) => {
            if *extension == std::ffi::OsStr::new("mp3") {
                Some(path.to_path_buf())
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn get_all_mp3_files_in_directory(directory: &Path) -> Vec<PathBuf> {
    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| mp3_file_paths(&e))
        .collect()
}

/* 
    TODO: 
        - Offer an option to Read the ID3 version as well as convert ID3v1 to ID3v2.3 or ID3v2.4
        
        - Need to DRY this out and change the logic.
            - Try to Parse as ID32.4 first
            - If Fails, try to parse as ID3v1
            - Read / Write appropriately
            - Auto Convert ID3v1 to ID3v2.4 on all writes? (Provide override?)
            - 
        
*/
pub fn process_file(args: &Cli, path: &PathBuf) {
    let possible_tag = Tag::read_from_path(&path);

    match possible_tag {
        Ok(mut tag) => match &args.cmd {
            Command::Read {
                artist,
                album,
                title,
                year,
            } => {
                if *artist {
                    match tag.artist() {
                        Some(artist) => println!("Artist: {}", artist),
                        None => println!("Artist: --"),
                    }
                }

                if *album {
                    match tag.album() {
                        Some(album) => println!("Album: {}", album),
                        None => println!("Album: --"),
                    }
                }

                if *title {
                    match tag.title() {
                        Some(title) => println!("Title: {}", title),
                        None => println!("Title: --"),
                    }
                }

                if *year {
                    match tag.year() {
                        Some(year) => println!("Year: {}", year),
                        None => println!("Year: --"),
                    }
                }

                println!("----------------")
            }
            Command::Write {
                artist,
                album,
                title,
                year,
            } => {
                if artist.is_some() {
                    tag.set_artist(artist.clone().unwrap());
                }

                if album.is_some() {
                    tag.set_album(album.clone().unwrap());
                }

                if title.is_some() {
                    tag.set_title(title.clone().unwrap());
                }

                if year.is_some() {
                    tag.set_year(year.unwrap());
                }

                println!("Writing to {:?}", &path);
                tag.write_to_path(&path, Version::Id3v24).unwrap();
            }
        },
        Err(_err) => {
            // Check if the error is instead related to parsing a ID3v1 tag.
            let mut file = std::fs::File::open(path).unwrap();
            let id3_v1_tag = id3::v1::Tag::read_from(&file);

            dbg!(&id3_v1_tag);
            let mut new_tag = Tag::new();

            if id3_v1_tag.is_ok() {
                let id3_v1_tag = id3_v1_tag.unwrap();
                let removed_tag = id3::v1::Tag::remove(&mut file);

                match removed_tag {
                    Ok(_) => {
                        // Now we must convert from Id3v1 to Id3v2.4
                        println!("Removed Id3v1 tag from {:?}", path);
                        match id3_v1_tag.genre() {
                            Some(genre) => {
                                println!("Genre: {:?}", genre);
                                new_tag.set_genre(genre);
                            }
                            None => {}
                        }

                        new_tag.set_artist(id3_v1_tag.artist);
                        new_tag.set_title(id3_v1_tag.title);
                        new_tag.set_album(id3_v1_tag.album);
                        new_tag.set_year(id3_v1_tag.year.parse::<i32>().unwrap());

                        match id3_v1_tag.track {
                            Some(track_num) => {
                                new_tag.set_track(track_num as u32);
                            }
                            None => {}
                        }

                        println!("Comment: {:?}", id3_v1_tag.comment);
                        new_tag.write_to_path(&path, Version::Id3v24).unwrap();
                        println!("Converted Id3v1 tag to Id3v2.4")
                    }
                    Err(err) => println!("Error removing Id3v1 Tag from {:?}: {:?}", path, err),
                }
            }

            match &args.cmd {
                Command::Write {
                    artist,
                    album,
                    title,
                    year,
                } => {
                    if artist.is_some() {
                        new_tag.set_artist(artist.clone().unwrap());
                    }

                    if album.is_some() {
                        new_tag.set_album(album.clone().unwrap());
                    }

                    if title.is_some() {
                        new_tag.set_title(title.clone().unwrap());
                    }

                    if year.is_some() {
                        new_tag.set_year(year.unwrap());
                    }

                    println!("Writing to {:?}", &path);
                    new_tag.write_to_path(&path, Version::Id3v24).unwrap();
                }
                _ => println!("Error parsing tag"),
            }
        }
    }
}
