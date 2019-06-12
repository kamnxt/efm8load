use std::io::Read;
use std::{fmt, fs, io};
pub type Efm8Cmd = Vec<u8>;
pub type Efm8Cmds = Vec<Efm8Cmd>;

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

pub fn parse_cmds(path: &str) -> Result<Efm8Cmds, ParseError> {
    let f = fs::File::open(path)?;
    let mut cmds: Efm8Cmds = Efm8Cmds::new();
    let iter = &mut f.bytes().peekable();
    println!("Parsing commands");
    loop {
        if let Some(Ok(0x24u8)) = iter.peek() {
            print!(".");
            let mut cmd: Efm8Cmd = Efm8Cmd::new();
            cmd.push(iter.next().unwrap().unwrap()); // $
            let num_bytes = iter.next().unwrap().unwrap();
            cmd.push(num_bytes); // num bytes
            cmd.extend(iter.by_ref().take(num_bytes as usize).map(|c| c.unwrap())); // command
            cmds.push(cmd);
            if let None = iter.peek() {
                break;
            }
        } else {
            eprintln!("Expected '$', found something else ({:?})", iter.peek());
            return Err(ParseError::ParseFailed);
        }
    }
    println!("done.");
    Ok(cmds)
}