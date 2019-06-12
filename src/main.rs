extern crate clap;
extern crate hidapi;

use clap::{App, Arg};
use std::fmt;
use std::fmt::Display;

use hidapi::HidApi;

mod parser;
mod types;
mod uploader;
use parser::parse_cmds;
use types::Config;
use uploader::upload_cmds;

const DEFAULT_VID: &str = "0x10c4";
const DEFAULT_PID: &str = "0xeac9";

fn main() {
    let config = App::new("efm8load-rs")
        .author("Kamil Krzyżanowski (kamnxt) <kamnxt@kamnxt.com>")
        .about("Loads a efm8 boot file onto an efm8 using the HID bootloader.")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("INPUT").help("File to load").required(true))
        .arg(
            Arg::with_name("PID")
                .short("p")
                .long("pid")
                .help("The product ID of the device")
                .default_value(DEFAULT_PID),
        )
        .arg(
            Arg::with_name("VID")
                .short("v")
                .long("vid")
                .help("The vendor ID of the device")
                .default_value(DEFAULT_VID),
        )
        .get_matches();
    match run(config) {
        Ok(()) => println!("Upload successful."),
        Err(err) => eprintln!("Upload failed due to an error: {}", err),
    }
}

enum AppError {
    ArgsError(std::num::ParseIntError),
    ParseError(parser::ParseError),
    HidApiError(hidapi::HidError),
    UploadError(uploader::UploadError),
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::ArgsError(err) => write!(f, "Invalid arguments: {}", err),
            AppError::ParseError(err) => write!(f, "Failed to parse efm8 file: {}", err),
            AppError::HidApiError(err) => write!(f, "Failed to open hidapi: {}", err),
            AppError::UploadError(err) => write!(f, "Failed to upload to device: {}", err),
        }
    }
}

impl From<hidapi::HidError> for AppError {
    fn from(err: hidapi::HidError) -> AppError {
        AppError::HidApiError(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> AppError {
        AppError::ArgsError(err)
    }
}

impl From<parser::ParseError> for AppError {
    fn from(err: parser::ParseError) -> AppError {
        AppError::ParseError(err)
    }
}

impl From<uploader::UploadError> for AppError {
    fn from(err: uploader::UploadError) -> AppError {
        AppError::UploadError(err)
    }
}

fn run(config: clap::ArgMatches) -> Result<(), AppError> {
    let config = verify_args(config)?;
    println!("Starting...");
    let cmds = parse_cmds(&config.path)?;
    let api = HidApi::new()?;
    upload_cmds(cmds, api, config.vid, config.pid)?;
    Ok(())
}

fn verify_args(matches: clap::ArgMatches) -> Result<Config, std::num::ParseIntError> {
    let path = matches.value_of("INPUT").unwrap().to_owned();
    let pid = u16::from_str_radix(
        matches.value_of("PID").unwrap().trim_start_matches("0x"),
        16,
    )?;
    let vid = u16::from_str_radix(
        matches.value_of("VID").unwrap().trim_start_matches("0x"),
        16,
    )?;
    Ok(Config { path, pid, vid })
}
