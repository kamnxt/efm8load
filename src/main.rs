extern crate hidapi;

use hidapi::HidApi;
use std::fs;
use std::io;
use std::io::Read;

const VID: u16 = 0x0000; // CHANGE ME TODO
const PID: u16 = 0x0000; // CHANGE ME TODO

fn upload_code(path: &str, api: HidApi) -> io::Result<()> {
    let mut f = fs::File::open(path)?;
    let mut cmds: Vec<Vec<u8>> = Vec::new();
    //    let mut iter = f.bytes().peekable();
    //    loop {
    //        if let Some(Ok(0x24u8)) = iter.peek() {
    //            println!(".");
    //            let arr: Vec<u8> = iter.by_ref().take(2).map(|c| c.unwrap()).collect();
    //            if let None = iter.peek() {
    //                break;
    //            }
    //            cmds.push(arr);
    //            iter.next();
    //        } else {
    //            eprintln!("Expected '$', found something else");
    //            return Ok(());
    //        }
    //    }
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut pos = 0;
    while pos < buf.len() {
        if buf[pos] != '$' as u8 {
            eprintln!("File seems weird");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                io::Error::last_os_error(),
            ));
        }
        let len = buf[pos + 1] as usize;
        let cmd = buf[pos..pos + len + 2].to_vec();
        cmds.push(cmd);
        pos += len + 2;
    }
    let dev = api.open(VID, PID);
    match dev {
        Err(idk) => {
            eprintln!("oof {}", idk);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                io::Error::last_os_error(),
            ));
        }
        Ok(dev) => {
            println!("Writing");
            for cmd in cmds {
                print!(".");
                dev.write(cmd.as_ref()).unwrap();
                let mut buf = [0u8];
                dev.read_timeout(&mut buf, 100).unwrap();
                if buf[0] != 0x40 {
                    eprintln!("Received {}", buf[0]);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        io::Error::last_os_error(),
                    ));
                }
            }
            println!(" and done!");
        }
    }
    println!("Everything went well!");
    Ok(())
}

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    println!("Starting...");
    if let Ok(api) = HidApi::new() {
        if let Err(err) = upload_code(path.as_ref(), api) {
            eprintln!("Failed to upload: {}.", err);
        } else {
            println!("Upload successful!");
        }
    } else {
        eprintln!("Failed to open hidapi.")
    }
}
