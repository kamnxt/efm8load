use crate::error::ParseError;
use crate::types::{Efm8Cmd, Efm8Cmds};
use std::fs;
use std::io::Read;

pub fn parse_cmds(path: &str) -> Result<Efm8Cmds, ParseError> {
    let f = fs::File::open(path)?;
    let mut cmds: Efm8Cmds = Efm8Cmds::new();
    let iter = &mut f.bytes().peekable();
    loop {
        if let Some(Ok(0x24u8)) = iter.peek() {
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
    println!("Found {} commands to send.", cmds.len());
    Ok(cmds)
}
