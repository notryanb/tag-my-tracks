extern crate id3;
extern crate failure;
extern crate quicli;
extern crate structopt;
extern crate walkdir;


use id3::{Tag, Version};
use failure::Error;
use quicli::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

pub mod cli;

use cli::{Cli, Command, ReadFields, WriteFields};

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
    // let id3v2_tag = Tag::read_from_path(&path);

    match &args.cmd {
        Command::Read(tag_fields) => read_tag_with_args(&tag_fields, &path),
        Command::Write(tag_fields) => write_tag_with_args(&tag_fields, &path),
    };

    // match id3v2_tag {
    //     Ok(mut tag) => match &args.cmd {
    //         Command::Read(tag_fields) => read_tag_with_args(&tag, &tag_fields),
    //         Command::Write(tag_fields) => write_tag_with_args(&mut tag, &tag_fields, &path),
    //     },
    //     Err(_err) => { // Can't parse an ID3v2.X tag from the path.
    //         // Check if the error is instead related to parsing a ID3v1 tag.
    //         let mut file = std::fs::File::open(path).unwrap();
    //         let id3_v1_tag = id3::v1::Tag::read_from(&file);

    //         dbg!(&id3_v1_tag);
    //         let mut new_tag = Tag::new();

    //         // Logic for converting / writing ID3v1 to ID3v2.4
    //         if id3_v1_tag.is_ok() {
    //             let id3_v1_tag = id3_v1_tag.unwrap();
    //             let removed_tag = id3::v1::Tag::remove(&mut file);

    //             match removed_tag {
    //                 Ok(_) => {
    //                     // Now we must convert from Id3v1 to Id3v2.4
    //                     println!("Removed Id3v1 tag from {:?}", path);
    //                     match id3_v1_tag.genre() {
    //                         Some(genre) => {
    //                             println!("Genre: {:?}", genre);
    //                             new_tag.set_genre(genre);
    //                         }
    //                         None => {}
    //                     }

    //                     new_tag.set_artist(id3_v1_tag.artist);
    //                     new_tag.set_title(id3_v1_tag.title);
    //                     new_tag.set_album(id3_v1_tag.album);
    //                     new_tag.set_year(id3_v1_tag.year.parse::<i32>().unwrap());

    //                     match id3_v1_tag.track {
    //                         Some(track_num) => {
    //                             new_tag.set_track(track_num as u32);
    //                         }
    //                         None => {}
    //                     }

    //                     println!("Comment: {:?}", id3_v1_tag.comment);
    //                     let convert_tag_result = new_tag.write_to_path(&path, Version::Id3v24);
    //                     match convert_tag_result {
    //                         Ok(_) => {
    //                             println!("Successfully converted {:?} form v1 tag to v2.4", &path)
    //                         }
    //                         Err(err) => println!("Error converting from v1 tag to v2.4: {:?}", err),
    //                     }

    //                     println!("Converted Id3v1 tag to Id3v2.4")
    //                 }

    //                 Err(err) => println!("Error removing Id3v1 Tag from {:?}: {:?}", path, err),
    //             }
    //         }

    //         // match &args.cmd {
    //         //     Command::Write(tag_fields) => {
    //         //         if tag_fields.artist.is_some() {
    //         //             new_tag.set_artist(tag_fields.artist.clone().unwrap());
    //         //         }

    //         //         if tag_fields.album.is_some() {
    //         //             new_tag.set_album(tag_fields.album.clone().unwrap());
    //         //         }

    //         //         if tag_fields.title.is_some() {
    //         //             new_tag.set_title(tag_fields.title.clone().unwrap());
    //         //         }

    //         //         if tag_fields.year.is_some() {
    //         //             new_tag.set_year(tag_fields.year.unwrap());
    //         //         }

    //         //         println!("Writing to {:?}", &path);
    //         //         new_tag.write_to_path(&path, Version::Id3v24).unwrap();
    //         //     }
    //         //     _ => println!("Error parsing tag"),
    //         // }
    //     }
    // }
}

#[derive(Debug, Fail)]
pub enum TagParseError {
    #[fail(display = "Couldn't parse to ID3v2")]
    InvalidVersion2Tag,
}

pub fn read_tag_with_args(fields: &ReadFields, path: &PathBuf) -> Result<(), Error> {
    let id3v2_tag = Tag::read_from_path(path);

    match id3v2_tag {
        Ok(tag) => {            
            if fields.artist {
                match tag.artist() {
                    Some(artist) => println!("Artist: {}", artist),
                    None => println!("Artist: --"),
                }
            }

            if fields.album {
                match tag.album() {
                    Some(album) => println!("Album: {}", album),
                    None => println!("Album: --"),
                }
            }

            if fields.title {
                match tag.title() {
                    Some(title) => println!("Title: {}", title),
                    None => println!("Title: --"),
                }
            }

            if fields.year {
                match tag.year() {
                    Some(year) => println!("Year: {}", year),
                    None => println!("Year: --"),
                }
            }

            println!("----------------");

        },
        Err(err) => {
            if !fields.convert {
                return Err(TagParseError::InvalidVersion2Tag);
            }
            println!("Error converting file to ID3v2.4");
            // let mut file = std::fs::File::open(path).unwrap();
            // let id3_v1_tag = id3::v1::Tag::read_from(&file);

            // dbg!(&id3_v1_tag);
            // let mut new_tag = Tag::new();
        }
    }

    Ok(())
}

pub fn write_tag_with_args(fields: &WriteFields, path: &PathBuf) -> Result<(), Error> {
    let id3v2_tag = Tag::read_from_path(path);

    match id3v2_tag {
        Ok(tag) => { 
            if fields.artist.is_some() {
                tag.set_artist(fields.artist.clone().unwrap());
            }

            if fields.album.is_some() {
                tag.set_album(fields.album.clone().unwrap());
            }

            if fields.title.is_some() {
                tag.set_title(fields.title.clone().unwrap());
            }

            if fields.year.is_some() {
                tag.set_year(fields.year.unwrap());
            }

            println!("Writing to {:?}", &path);
            tag.write_to_path(&path, Version::Id3v24)?;
        },
        Err(err) => {
            if !fields.convert {
                return Err(TagParseError::InvalidVersion2Tag);
            }
            println!("Error converting file to ID3v2.4");
        }
    }

    Ok(())
}