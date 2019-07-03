use std::fmt::Display;
use std::{fmt, io};

pub enum AppError {
    ArgsError(std::num::ParseIntError),
    ParseError(ParseError),
    HidApiError(hidapi::HidError),
    UploadError(UploadError),
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

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> AppError {
        AppError::ParseError(err)
    }
}

impl From<UploadError> for AppError {
    fn from(err: UploadError) -> AppError {
        AppError::UploadError(err)
    }
}

pub enum ParseError {
    ParseFailed,
    IoError(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ParseError::ParseFailed => write!(f, "could not parse file"),
            ParseError::IoError(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> ParseError {
        ParseError::IoError(error)
    }
}

pub enum Efm8Error {
    BadCRC,
    BadID,
    Range,
    Other(u8),
}

impl Efm8Error {
    pub fn from_value(value: u8) -> Efm8Error {
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
