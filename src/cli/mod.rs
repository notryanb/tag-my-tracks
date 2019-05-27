use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Command",
    about = "Read or write fields from the ID3 Tags for a given path."
)]
pub enum Command {
    #[structopt(
        name = "read",
        help = "Reads the requested fields from the ID3 tag(s) specified in the path"
    )]
    Read(ReadFields),

    #[structopt(
        name = "write",
        help = "Writes the requested fields and their values to ID3v2.4 tag(s) specified in the path"
    )]
    Write(WriteFields),
}

#[derive(Debug, StructOpt)]
pub struct ReadFields {
    #[structopt(long = "artist")]
    pub artist: bool,

    #[structopt(long = "album")]
    pub album: bool,

    #[structopt(long = "title")]
    pub title: bool,

    #[structopt(long = "year")]
    pub year: bool,
}

#[derive(Debug, StructOpt)]
pub struct WriteFields {
    #[structopt(long = "artist")]
    pub artist: Option<String>,

    #[structopt(long = "album")]
    pub album: Option<String>,

    #[structopt(long = "title")]
    pub title: Option<String>,

    #[structopt(long = "year")]
    pub year: Option<i32>,
}
