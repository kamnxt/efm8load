use crate::types::Efm8Cmds;
use hidapi::HidApi;
use std::fmt;

pub fn upload_cmds(cmds: Efm8Cmds, api: HidApi, pid: u16, vid: u16) -> Result<(), UploadError> {
    println!("{:x}, {:x}", vid, pid);
    println!("Writing");
    let dev = api.open(vid, pid)?;
    for cmd in cmds {
        dev.set_blocking_mode(true)?;
        print!("o");
        let step_size = 32;
        for i in (0..cmd.len()).step_by(step_size) {
            let mut buf = Vec::new();
            buf.push(0);
            let mut bytes_left = cmd.len() - i;
            if bytes_left > step_size {
                bytes_left = step_size
            }
            buf.extend(&cmd[i..i + bytes_left]);
            dev.write(buf.as_slice())?;
        }

        let mut buf = [0u8; 4];
        match dev.read_timeout(&mut buf[..], 200) {
            Ok(4) => {
                print!("\u{8}."); // backspace and a dot
                if buf[0] != 0x40 {
                    return Err(UploadError::LoadFailed(Efm8Error::from_value(buf[0])));
                }
            }
            Ok(0) => {
                return Err(UploadError::Timeout);
            }
            Err(error) => {
                println!("Failed to read");
                return Err(UploadError::HidError(error));
            }
            Ok(bytes) => {
                //this should never happen
                println!("Read {} bytes!", bytes)
            }
        }
    }
    println!(" and done!");
    println!("Everything went well!");
    Ok(())
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
pub enum UploadError {
    HidError(hidapi::HidError),
    LoadFailed(Efm8Error),
    Timeout,
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            UploadError::HidError(err) => write!(f, "communicating with device failed: {}", err),
            UploadError::LoadFailed(code) => write!(f, "device returned unknown code {}", code),
            UploadError::Timeout => write!(f, "read from device timed out"),
        }
    }
}

impl From<hidapi::HidError> for UploadError {
    fn from(error: hidapi::HidError) -> UploadError {
        UploadError::HidError(error)
    }
}
