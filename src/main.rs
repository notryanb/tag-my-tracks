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
        #[structopt(long = "artist")]
        artist: bool,

        #[structopt(long = "album")]
        album: bool,

        #[structopt(long = "title")]
        title: bool,

        #[structopt(long = "year")]
        year: bool,
    },

    #[structopt(name = "write")]
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

fn main() -> CliResult {
    let args = Cli::from_args();
    let path = &args.path;

    if !path.exists() {
        warn!("Error: {:?} is not a valid path", &path);
    }

    if path.is_file() {
        process_file(&args, &path);
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
                            },
                            None => {}
                        }

                        new_tag.set_artist(id3_v1_tag.artist);
                        new_tag.set_title(id3_v1_tag.title);
                        new_tag.set_album(id3_v1_tag.album);
                        new_tag.set_year(id3_v1_tag.year.parse::<i32>().unwrap());

                        match id3_v1_tag.track {
                            Some(track_num) => {
                                new_tag.set_track(track_num as u32);
                            },
                            None => {}
                        }

                        
                        println!("Comment: {:?}", id3_v1_tag.comment);
                        new_tag.write_to_path(&path, Version::Id3v24).unwrap();
                        println!("Converted Id3v1 tag to Id3v2.4")
                    },
                    Err(err) => println!("Error removing Id3v1 Tag from {:?}: {:?}", path, err)
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
