use id3::{Tag, Version};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

use tmt::{Cli, Command, ReadFields, WriteFields};

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();
    let path = &args.path;

    if !path.exists() {
        println!("Error: {:?} is not a valid path", &path);
    }

    if path.is_file() {
        process_file(&args, &path)?;
    } else {
        let mp3_paths = get_all_mp3_files_in_directory(&args.path);
        for path in mp3_paths.into_iter() {
            process_file(&args, &path)?;
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
pub fn process_file(args: &Cli, path: &PathBuf) -> Result<(), TagParseError> {
    match &args.cmd {
        Command::Read(tag_fields) => read_tag_with_args(&tag_fields, &path)?,
        Command::Write(tag_fields) => write_tag_with_args(&tag_fields, &path)?,
    };

    Ok(())
}

pub fn read_tag_with_args(fields: &ReadFields, path: &PathBuf) -> Result<(), TagParseError> {
    let id3v2_tag = Tag::read_from_path(path);

    println!("Fields: {:?}", &fields);

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
        }
        Err(err) => {
            let file = std::fs::File::open(path).unwrap();
            let id3_v1_tag = id3::v1::Tag::read_from(&file);

            if fields.convert {
                match id3_v1_tag {
                    Ok(tag) => {
                        if fields.artist {
                            println!("Artist: {}", tag.artist);
                        }

                        if fields.album {
                            println!("Album: {}", tag.album);
                        }

                        if fields.title {
                            println!("Title: {}", tag.title);
                        }

                        if fields.year {
                            println!("Year: {}", tag.year);
                        }

                        println!("----------------");
                        return Ok(());
                    }
                    Err(err) => {
                        return Err(TagParseError::InvalidVersion2Tag(
                            err.description.to_string(),
                        ));
                    }
                }
            }

            return Err(TagParseError::InvalidVersion2Tag(
                err.description.to_string(),
            ));
        }
    }

    Ok(())
}

pub fn write_tag_with_args(fields: &WriteFields, path: &PathBuf) -> Result<(), TagParseError> {
    let id3v2_tag = Tag::read_from_path(path);

    match id3v2_tag {
        Ok(mut tag) => {
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
        }
        Err(err) => {
            if !fields.convert {
                return Err(TagParseError::InvalidVersion2Tag(
                    err.description.to_string(),
                ));
            }

            let file = std::fs::File::open(path).unwrap();
            let id3_v1_tag = id3::v1::Tag::read_from(&file);

            let mut new_tag = Tag::new();

            println!("Converting {:?} to Id3v2.4", &path);

            match id3_v1_tag {
                Ok(tag) => {
                    new_tag.set_artist(tag.artist);
                    new_tag.set_album(tag.album);
                    new_tag.set_title(tag.title);
                    new_tag.set_year(tag.year.parse::<i32>()?);
                    new_tag.write_to_path(&path, Version::Id3v24)?;
                }
                Err(err) => {
                    return Err(TagParseError::InvalidVersion2Tag(
                        err.description.to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum TagParseError {
    InvalidVersion2Tag(String),
}

impl std::fmt::Display for TagParseError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            TagParseError::InvalidVersion2Tag(ref f) => {
                write!(fmt, "Couldn't parse ID3v2 tag from  `{}`", f)
            }
        }
    }
}

impl std::error::Error for TagParseError {
    fn description(&self) -> &str {
        match *self {
            TagParseError::InvalidVersion2Tag(..) => "Failed to parse tag",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            _ => None,
        }
    }
}

impl From<std::io::Error> for TagParseError {
    fn from(err: std::io::Error) -> TagParseError {
        use std::error::Error;
        TagParseError::InvalidVersion2Tag(err.description().to_string())
    }
}

impl From<TagParseError> for std::io::Error {
    fn from(_err: TagParseError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, "There was an error")
    }
}

impl From<id3::Error> for TagParseError {
    fn from(err: id3::Error) -> TagParseError {
        TagParseError::InvalidVersion2Tag(err.description.to_string())
    }
}

impl From<std::num::ParseIntError> for TagParseError {
    fn from(_err: std::num::ParseIntError) -> TagParseError {
        TagParseError::InvalidVersion2Tag(String::from(
            "Failed to parse year from id3v1 tag while converting",
        ))
    }
}
