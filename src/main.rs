extern crate clap;
extern crate hidapi;

use clap::{App, Arg};

use hidapi::HidApi;
use std::fmt;

use std::thread;
use std::time::Duration;

mod parser;
use parser::parse_cmds;
use parser::Efm8Cmds;

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
                .default_value("0x0000"),
        )
        .get_matches();
    let path = config.value_of("INPUT").unwrap();
    let pid = config.value_of("PID").unwrap().trim_start_matches("0x");
    if let Ok(pid) = u16::from_str_radix(pid, 16) {
        println!("Starting...");
        let res = parse_cmds(path);
        match res {
            Ok(cmds) => {
                if let Ok(api) = HidApi::new() {
                    if let Err(err) = upload_cmds(cmds, api, pid) {
                        eprintln!("Failed to upload: {}.", err);
                    } else {
                        println!("Upload successful!");
                    }
                } else {
                    eprintln!("Failed to open hidapi.")
                }
            }
            Err(err) => {
                eprintln!("Failed to load commands from {}: {}", path, err);
            }
        }
    } else {
        eprintln!("Failed to parse PID.");
    }
}

#[derive(Debug)]
pub enum Efm8Error {
    BadCRC,
    BadID,
    Range,
    Other(u8),
}

impl Efm8Error {
    fn from_value(value: u8) -> Efm8Error {
        match value {
            0x41 => Efm8Error::Range,
            0x42 => Efm8Error::BadID,
            0x43 => Efm8Error::BadCRC,
            other => Efm8Error::Other(other),
        }
    }
}

impl fmt::Display for Efm8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Efm8Error::Range => write!(f, "bad range"),
            Efm8Error::BadCRC => write!(f, "bad CRC"),
            Efm8Error::BadID => write!(f, "device id didn't match"),
            Efm8Error::Other(byte) => write!(f, "device returned unknown code: {:x}", byte),
        }
    }
}

#[derive(Debug)]
enum Efm8LoadError {
    HidError(hidapi::HidError),
    LoadFailed(Efm8Error),
    Timeout,
}

impl fmt::Display for Efm8LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Efm8LoadError::HidError(err) => err.fmt(f),
            Efm8LoadError::LoadFailed(code) => write!(f, "device returned unknown code {}", code),
            Efm8LoadError::Timeout => write!(f, "read from device timed out"),
        }
    }
}

impl From<hidapi::HidError> for Efm8LoadError {
    fn from(error: hidapi::HidError) -> Efm8LoadError {
        Efm8LoadError::HidError(error)
    }
}

const VID: u16 = 0x10c4; // CHANGE ME TODO
const DEFAULT_PID: &str = "0xeac9"; // CHANGE ME TODO

fn upload_cmds(cmds: Efm8Cmds, api: HidApi, pid: u16) -> Result<(), Efm8LoadError> {
    println!("{:x}, {:x}", VID, pid);
    println!("Writing");
    for cmd in cmds {
        let dev = api.open(VID, pid)?;
        dev.set_blocking_mode(true)?;
        print!(".");
        let step_size = 32;
        for i in (0..cmd.len()).step_by(step_size) {
            let mut buf = Vec::new();
            buf.push(0);
            let mut bytes_left = cmd.len() - i;
            if bytes_left > step_size {
                bytes_left = step_size
            }
            buf.extend(&cmd[i..i + bytes_left]);
            dev.write(buf.as_slice()).expect("write");
        }

        let mut buf = [0u8; 200];
        match dev.read_timeout(&mut buf[..], 200) {
            Ok(4) => {
                if buf[0] != 0x40 {
                    return Err(Efm8LoadError::LoadFailed(Efm8Error::from_value(buf[0])));
                }
            }
            Ok(0) => {
                return Err(Efm8LoadError::Timeout);
            }
            Err(error) => {
                println!("Failed to read");
                return Err(Efm8LoadError::HidError(error));
            }
            Ok(bytes) => {
                //this should never happen
                println!("Read {} bytes into a 1 byte buffer!", bytes)
            }
        }
    }
    println!(" and done!");
    println!("Everything went well!");
    Ok(())
}
