use crate::error::{Efm8Error, UploadError};
use crate::types::Efm8Cmds;
use hidapi::HidApi;

pub fn upload_cmds(cmds: Efm8Cmds, api: HidApi, vid: u16, pid: u16) -> Result<(), UploadError> {
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
